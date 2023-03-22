//! What are units attempting to achieve?

use bevy::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::{thread_rng, Rng};

use crate::asset_management::manifest::{
    Id, Item, ItemManifest, Structure, StructureManifest, Unit, UnitManifest,
};
use crate::signals::{SignalType, Signals};
use crate::simulation::geometry::TilePos;

use super::impatience::ImpatiencePool;

/// A unit's current goals.
///
/// Units will be fully concentrated on any task other than [`Goal::Wander`] until it is complete (or overridden).
/// Once a goal is complete, they will typically transition back into [`Goal::Wander`] and attempt to find something new to do.
///
/// This component serves as a state machine.
#[derive(Component, PartialEq, Eq, Clone, Default, Debug)]
pub(crate) enum Goal {
    /// Attempting to find something useful to do
    ///
    /// Units will try and follow a signal, if they can pick up a trail, but will not fixate on it until the signal is strong enough.
    #[default]
    Wander,
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

impl TryFrom<SignalType> for Goal {
    // At least for now, this conversion should never fail.
    type Error = ();

    fn try_from(value: SignalType) -> Result<Goal, Self::Error> {
        match value {
            // Go grab the item, so you can later take it away
            SignalType::Push(item_id) => Ok(Goal::Pickup(item_id)),
            // Go grab the item, so you can bring it to me
            SignalType::Pull(item_id) => Ok(Goal::Pickup(item_id)),
            SignalType::Contains(_) => Err(()),
            SignalType::Stores(_) => Err(()),
            SignalType::Work(structure_id) => Ok(Goal::Work(structure_id)),
            SignalType::Demolish(structure_id) => Ok(Goal::Demolish(structure_id)),
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
            Goal::Wander => "Wander".to_string(),
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
    mut units_query: Query<(&TilePos, &mut Goal, &mut ImpatiencePool, &Id<Unit>)>,
    unit_manifest: Res<UnitManifest>,
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();

    for (&tile_pos, mut goal, mut impatience_pool, id) in units_query.iter_mut() {
        // If we're out of patience, give up and choose a new goal
        if impatience_pool.is_full() {
            *goal = Goal::Wander;
        }

        // By default, goals are reset to wandering when completed.
        // Pick a new goal when wandering.
        // If anything fails, just keep wandering for now.
        if let Goal::Wander = *goal {
            let unit_data = unit_manifest.get(*id);
            // Continuing wandering for longer increases exploration, and allows units to spread out across the map more readily.
            if rng.gen_bool(1. / unit_data.mean_free_wander_period) {
                return;
            }

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
                    *goal = selected_signal_type.try_into().unwrap();
                    // Reset impatience when we choose a new goal
                    impatience_pool.reset();
                }
            }
        }
    }
}
