//! What are units attempting to achieve?

use bevy::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::thread_rng;

use crate::asset_management::manifest::{
    Id, Item, ItemManifest, Structure, StructureManifest, Unit, UnitManifest,
};
use crate::signals::{SignalType, Signals};
use crate::simulation::geometry::TilePos;

use super::impatience::ImpatiencePool;
use super::item_interaction::UnitInventory;
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
    /// Attempting to pick up an object
    #[allow(dead_code)]
    Pickup(Id<Item>),
    /// Attempting to drop off an object
    ///
    /// This can place the object in storage or a structure that actively needs it.
    #[allow(dead_code)]
    Store(Id<Item>),
    /// Attempting to drop off an object to a structure that actively needs it.
    #[allow(dead_code)]
    Deliver(Id<Item>),
    /// Attempting to perform work at a structure
    #[allow(dead_code)]
    Work(Id<Structure>),
    /// Attempt to feed self
    Eat(Id<Item>),
    /// Attempting to destroy a structure
    Demolish(Id<Structure>),
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
            SignalType::Push(item_id) => Ok(Goal::Pickup(item_id)),
            // Go grab the item, so you can bring it to me
            SignalType::Pull(item_id) => Ok(Goal::Pickup(item_id)),
            SignalType::Work(structure_id) => Ok(Goal::Work(structure_id)),
            SignalType::Demolish(structure_id) => Ok(Goal::Demolish(structure_id)),
            SignalType::Contains(_) => Err(()),
            SignalType::Stores(_) => Err(()),
            SignalType::Unit(_) => Err(()),
        }
    }
}

impl Goal {
    /// Pretty formatting for this type
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
    ) -> String {
        match self {
            Goal::Wander { remaining_actions } => format!(
                "Wander ({} actions remaining)",
                remaining_actions.unwrap_or(0)
            ),
            Goal::Pickup(item) => format!("Pickup {}", item_manifest.name(*item)),
            Goal::Store(item) => format!("Store {}", item_manifest.name(*item)),
            Goal::Deliver(item) => format!("Deliver {}", item_manifest.name(*item)),
            Goal::Work(structure) => format!("Work at {}", structure_manifest.name(*structure)),
            Goal::Demolish(structure) => {
                format!("Demolish {}", structure_manifest.name(*structure))
            }
            Goal::Eat(item) => format!("Eat {}", item_manifest.name(*item)),
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
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();

    for (&tile_pos, mut goal, mut impatience_pool, unit_inventory, id) in units_query.iter_mut() {
        // If we're out of patience, give up and choose a new goal
        if impatience_pool.is_full() {
            // If you're holding something, try to put it away nicely
            *goal = if let Some(held_item) = unit_inventory.held_item {
                // Don't get stuck trying to do a hopeless storage task forever
                if !matches!(*goal, Goal::Store(..) | Goal::Wander { .. }) {
                    Goal::Store(held_item)
                } else {
                    Goal::Wander {
                        remaining_actions: None,
                    }
                }
            } else {
                Goal::Wander {
                    remaining_actions: None,
                }
            };

            // Reset impatience when we choose a new goal
            impatience_pool.reset();
        }

        if let Goal::Wander { remaining_actions } = *goal {
            let wandering_behavior = &unit_manifest.get(*id).wandering_behavior;
            *goal = compute_new_goal(
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
    mut remaining_actions: Option<u16>,
    tile_pos: TilePos,
    wandering_behavior: &WanderingBehavior,
    rng: &mut ThreadRng,
    signals: &Signals,
) -> Goal {
    // When we first get a wandering goal, pick a number of actions to take before picking a new goal.
    if remaining_actions.is_none() {
        remaining_actions = Some(wandering_behavior.sample(rng));
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
    if let Ok(goal_weights) = WeightedIndex::new(
        goal_relevant_signals
            .clone()
            .map(|(_type, strength)| strength.value()),
    ) {
        let selected_goal_index = goal_weights.sample(rng);
        if let Some(selected_signal) = goal_relevant_signals.nth(selected_goal_index) {
            let selected_signal_type = *selected_signal.0;
            selected_signal_type.try_into().unwrap()
        } else {
            Goal::Wander { remaining_actions }
        }
    } else {
        Goal::Wander { remaining_actions }
    }
}
