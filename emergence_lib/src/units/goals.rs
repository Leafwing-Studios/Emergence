//! What are units attempting to achieve?

use bevy::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::thread_rng;

use crate::asset_management::manifest::Id;
use crate::construction::ghosts::WorkplaceId;
use crate::crafting::item_tags::ItemKind;
use crate::geometry::TilePos;
use crate::items::item_manifest::ItemManifest;
use crate::signals::{SignalType, Signals};
use crate::structures::structure_manifest::{Structure, StructureManifest};
use crate::terrain::terrain_manifest::TerrainManifest;

use super::actions::{DeliveryMode, Purpose};
use super::impatience::ImpatiencePool;
use super::item_interaction::UnitInventory;
use super::unit_manifest::{Unit, UnitManifest};
use super::WanderingBehavior;

/// A unit's current goals.
///
/// Units will be fully concentrated on any task other than [`Goal::Wander`] until it is complete (or overridden).
/// Once a goal is complete, they will typically transition back into [`Goal::Wander`] and attempt to find something new to do.
///
/// This component serves as a state machine.
#[derive(Component, PartialEq, Clone, Debug)]
pub(crate) enum Goal {
    /// Attempting to find something useful to do
    ///
    /// Units will try and follow a signal, if they can pick up a trail, but will not fixate on it until the signal is strong enough.
    Wander {
        /// How many actions will this unit take before picking a new goal?
        remaining_actions: Option<u16>,
    },
    /// Attempting to pick up an object, so it can be taken away from a structure that actively rejects it.
    ///
    /// This is [`DeliveryMode::PickUp`] and [`Purpose::Intrinsic`].
    Remove(ItemKind),
    /// Attempting to pick up an object wherever we can, so it can be delivered to a structure.
    ///
    /// This is [`DeliveryMode::PickUp`] and [`Purpose::Instrumental`].
    Fetch(ItemKind),
    /// Attempting to drop off an object to a structure that actively needs it.
    ///
    /// This is [`DeliveryMode::DropOff`] and [`Purpose::Intrinsic`].
    Deliver(ItemKind),
    /// Attempting to drop off an object wherever we can.
    ///
    /// This is [`DeliveryMode::DropOff`] and [`Purpose::Instrumental`].
    Store(ItemKind),
    /// Attempting to perform work at a structure.
    Work(WorkplaceId),
    /// Attempting to destroy a structure.
    Demolish(Id<Structure>),
    /// Attempting to feed self.
    Eat(ItemKind),
    /// Attempting to get to oxygen.
    Breathe,
    /// Trying to avoid a specific unit.
    Avoid(Id<Unit>),
}

/// The data-less version of [`Goal`].
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum GoalKind {
    /// Attempting to find something useful to do.
    Wander,
    /// Attempting to pick up an object, so it can be taken away from a structure that actively rejects it.
    Remove,
    /// Attempting to pick up an object wherever we can, so it can be delivered to a structure.
    Fetch,
    /// Attempting to drop off an object to a structure that actively needs it.
    Deliver,
    /// Attempting to drop off an object wherever we can.
    Store,
    /// Attempting to perform work at a structure.
    Work,
    /// Attempting to destroy a structure.
    Demolish,
    /// Attempting to feed self.
    Eat,
    /// Trying to avoid a specific unit.
    Avoid,
    /// Trying to get to oxygen.
    Breathe,
}

impl From<&Goal> for GoalKind {
    fn from(value: &Goal) -> Self {
        match value {
            Goal::Wander { .. } => GoalKind::Wander,
            Goal::Remove(_) => GoalKind::Remove,
            Goal::Fetch(_) => GoalKind::Fetch,
            Goal::Deliver(_) => GoalKind::Deliver,
            Goal::Store(_) => GoalKind::Store,
            Goal::Work(_) => GoalKind::Work,
            Goal::Demolish(_) => GoalKind::Demolish,
            Goal::Eat(_) => GoalKind::Eat,
            Goal::Avoid(_) => GoalKind::Avoid,
            Goal::Breathe => GoalKind::Breathe,
        }
    }
}

impl Default for Goal {
    fn default() -> Self {
        Goal::Wander {
            remaining_actions: None,
        }
    }
}

impl TryFrom<SignalType> for Goal {
    // At least for now, this conversion should never fail.
    type Error = ();

    fn try_from(value: SignalType) -> Result<Goal, Self::Error> {
        match value {
            // Go grab the item, so you can later take it away
            SignalType::Push(item_kind) => Ok(Goal::Remove(item_kind)),
            // Go grab the item, so you can bring it to me
            SignalType::Pull(item_kind) => Ok(Goal::Fetch(item_kind)),
            SignalType::Work(structure_id) => Ok(Goal::Work(structure_id)),
            SignalType::Demolish(structure_id) => Ok(Goal::Demolish(structure_id)),
            SignalType::Contains(_) => Err(()),
            SignalType::Stores(_) => Err(()),
            SignalType::Unit(unit) => Ok(Goal::Avoid(unit)),
        }
    }
}

impl Goal {
    /// Returns whether the goal is to drop off an item, pick up an item or neither.
    pub(crate) fn delivery_mode(&self) -> Option<DeliveryMode> {
        match self {
            Goal::Wander { .. } => None,
            Goal::Remove(_) => Some(DeliveryMode::PickUp),
            Goal::Fetch(_) => Some(DeliveryMode::PickUp),
            Goal::Deliver(_) => Some(DeliveryMode::DropOff),
            Goal::Store(_) => Some(DeliveryMode::DropOff),
            Goal::Work(_) => None,
            Goal::Demolish(_) => None,
            Goal::Eat(_) => Some(DeliveryMode::PickUp),
            Goal::Avoid(_) => None,
            Goal::Breathe => None,
        }
    }

    /// Returns whether the goal is active or passive.
    pub(crate) fn purpose(&self) -> Purpose {
        match self {
            Goal::Wander { .. } => Purpose::Instrumental,
            Goal::Remove(_) => Purpose::Intrinsic,
            Goal::Fetch(_) => Purpose::Instrumental,
            Goal::Deliver(_) => Purpose::Intrinsic,
            Goal::Store(_) => Purpose::Instrumental,
            Goal::Work(_) => Purpose::Intrinsic,
            Goal::Demolish(_) => Purpose::Intrinsic,
            Goal::Eat(_) => Purpose::Instrumental,
            Goal::Breathe => Purpose::Instrumental,
            Goal::Avoid(_) => Purpose::Instrumental,
        }
    }

    /// Pretty formatting for this type
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
        terrain_manifest: &TerrainManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        match self {
            Goal::Wander { remaining_actions } => format!(
                "Wander ({} actions remaining)",
                remaining_actions.unwrap_or(0)
            ),
            Goal::Fetch(item_kind) => format!("Fetch {}", item_manifest.name_of_kind(*item_kind)),
            Goal::Remove(item_kind) => format!("Remove {}", item_manifest.name_of_kind(*item_kind)),
            Goal::Store(item_kind) => format!("Store {}", item_manifest.name_of_kind(*item_kind)),
            Goal::Deliver(item_kind) => {
                format!("Deliver {}", item_manifest.name_of_kind(*item_kind))
            }
            Goal::Work(workplace) => format!(
                "Work at {}",
                workplace.name(structure_manifest, terrain_manifest)
            ),
            Goal::Demolish(structure) => {
                format!("Demolish {}", structure_manifest.name(*structure))
            }
            Goal::Eat(item_kind) => format!("Eat {}", item_manifest.name_of_kind(*item_kind)),
            Goal::Avoid(unit) => format!("Avoid {}", unit_manifest.name(*unit)),
            Goal::Breathe => "Breathe".to_string(),
        }
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(
    mut units_query: Query<(
        &TilePos,
        &mut Goal,
        &mut ImpatiencePool,
        &UnitInventory,
        &Id<Unit>,
    )>,
    unit_manifest: Res<UnitManifest>,
    item_manifest: Res<ItemManifest>,
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();

    for (&tile_pos, mut goal, mut impatience_pool, unit_inventory, &unit_id) in
        units_query.iter_mut()
    {
        // If we're out of patience, give up and choose a new goal
        if impatience_pool.is_full() {
            // If you're holding something, try to put it away nicely
            *goal = if let Some(held_item) = unit_inventory.held_item {
                match &*goal {
                    Goal::Store(item_kind) | Goal::Deliver(item_kind) => {
                        // If we ran out of patience while trying to store something, we should just give up and drop it
                        if item_kind.matches(held_item, &item_manifest) {
                            Goal::Wander {
                                remaining_actions: None,
                            }
                        } else {
                            Goal::Store(ItemKind::Single(held_item))
                        }
                    }
                    _ => Goal::Store(ItemKind::Single(held_item)),
                }
            // If you're not holding anything, you can't store it!
            } else {
                Goal::Wander {
                    remaining_actions: None,
                }
            };

            // Reset impatience when we choose a new goal
            impatience_pool.reset();
        }

        if let Goal::Wander { remaining_actions } = *goal {
            let wandering_behavior = &unit_manifest.get(unit_id).wandering_behavior;
            *goal = compute_new_goal(
                unit_id,
                remaining_actions,
                tile_pos,
                wandering_behavior,
                rng,
                &signals,
            );

            // Reset impatience when we choose a new goal
            impatience_pool.reset();
        }
    }
}

/// Pick a new goal when wandering.
///
// By default, goals are reset to wandering when completed.
/// If anything fails, just keep wandering for now.
fn compute_new_goal(
    unit_id: Id<Unit>,
    mut remaining_actions: Option<u16>,
    tile_pos: TilePos,
    wandering_behavior: &WanderingBehavior,
    rng: &mut ThreadRng,
    signals: &Signals,
) -> Goal {
    // When we first get a wandering goal, pick a number of actions to take before picking a new goal.
    if remaining_actions.is_none() {
        let number_of_actions = wandering_behavior.sample(rng);
        remaining_actions = Some(number_of_actions);
    }

    // If we have actions left while wandering, use them up before picking a new goal.
    if let Some(n) = remaining_actions {
        if n != 0 {
            return Goal::Wander {
                remaining_actions: Some(n - 1),
            };
        }
    }

    // Pick a new goal based on the signals at this tile
    let current_signals = signals.all_signals_at_position(tile_pos);
    let mut goal_relevant_signals = current_signals.goal_relevant_signals();

    // Only try to avoid units of the same type
    goal_relevant_signals.retain(|(signal_type, _)| {
        if let SignalType::Unit(signal_unit_id) = signal_type {
            *signal_unit_id == unit_id
        } else {
            true
        }
    });

    if let Ok(goal_weights) = WeightedIndex::new(
        goal_relevant_signals
            .iter()
            .map(|(_type, strength)| strength.value()),
    ) {
        let selected_goal_index = goal_weights.sample(rng);
        if let Some(selected_signal) = goal_relevant_signals.get(selected_goal_index) {
            let selected_signal_type = *selected_signal.0;
            selected_signal_type.try_into().unwrap()
        } else {
            Goal::Wander { remaining_actions }
        }
    } else {
        Goal::Wander { remaining_actions }
    }
}
