//! What are units attempting to achieve?

use bevy::prelude::*;
use core::fmt::Display;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::thread_rng;

use crate::asset_management::manifest::{Id, Item, Structure};
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
    #[allow(dead_code)]
    DropOff(Id<Item>),
    /// Attempting to perform work at a structure
    #[allow(dead_code)]
    Work(Id<Structure>),
    /// Attempt to feed self
    Eat(Id<Item>),
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
            SignalType::Work(structure_id) => Ok(Goal::Work(structure_id)),
        }
    }
}

impl Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match self {
            Goal::Wander => "Wander".to_string(),
            Goal::Pickup(item) => format!("Pickup {item}"),
            Goal::DropOff(item) => format!("Dropoff {item}"),
            Goal::Work(structure) => format!("Work at {structure}"),
            Goal::Eat(item) => format!("Eat {item}"),
        };

        write!(f, "{string}")
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(
    mut units_query: Query<(&TilePos, &mut Goal, &mut ImpatiencePool)>,
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();

    for (&tile_pos, mut goal, mut impatience_pool) in units_query.iter_mut() {
        // If we're out of patience, give up and choose a new goal
        if impatience_pool.is_full() {
            *goal = Goal::Wander;
        }

        // By default, goals are reset to wandering when completed.
        // Pick a new goal when wandering.
        // If anything fails, just keep wandering for now.
        if let Goal::Wander = *goal {
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
