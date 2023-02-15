//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;
use rand::thread_rng;

use crate::items::ItemId;
use crate::organisms::units::UnitId;
use crate::simulation::geometry::{MapGeometry, TilePos};

/// A unit's current goals.
///
/// Units will be fully concentrated on any task other than [`CurrentGoal::Wander`] until it is complete (or overridden).
///
/// This component serves as a state machine.
#[derive(Component, PartialEq, Eq, Clone, Default)]
pub(crate) enum CurrentGoal {
    /// Attempting to find something useful to do
    ///
    /// Units will try and follow a signal, if they can pick up a trail, but will not fixate on it until the signal is strong enough.
    #[default]
    Wander,
    /// Attempting to pick up an object
    #[allow(dead_code)]
    Pickup(ItemId),
    /// Attempting to drop off an object
    #[allow(dead_code)]
    DropOff(ItemId),
    /// Attempting to perform work at a structure
    #[allow(dead_code)]
    Work(ItemId),
}

impl CurrentGoal {
    /// Get the interactable required for the unit to achieve its goal
    fn required_interactable(&self) -> Option<ItemId> {
        match self {
            CurrentGoal::Wander => None,
            CurrentGoal::Pickup(id) => Some(id.clone()),
            CurrentGoal::DropOff(id) => Some(id.clone()),
            CurrentGoal::Work(id) => Some(id.clone()),
        }
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(mut units_query: Query<(&UnitId, &mut CurrentGoal)>) {
    for (_unit, current_goal) in units_query.iter_mut() {
        // Check to see if any of the possible goals are high enough priority to swap to
        if *current_goal == CurrentGoal::Wander {
            //todo!()
        }
    }
}

/// Ticks the timer for each [`CurrentAction`].
pub(super) fn advance_action_timer(mut units_query: Query<&mut CurrentAction>, time: Res<Time>) {
    let delta = time.delta();

    for mut current_action in units_query.iter_mut() {
        current_action.timer.tick(delta);
    }
}

/// Choose the unit's action for this turn
pub(super) fn choose_action(
    mut units_query: Query<(&TilePos, &CurrentGoal, &mut CurrentAction), With<UnitId>>,
    map_geometry: Res<MapGeometry>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (unit_tile_pos, current_goal, mut current_action) in units_query.iter_mut() {
        if current_action.timer.finished() {
            if let Some(_required_interactable) = current_goal.required_interactable() {
                // TODO: check neighboring entities
            } else {
                if let Some(target_tile) =
                    unit_tile_pos.choose_random_empty_neighbor(rng, map_geometry)
                {
                    *current_action = CurrentAction {
                        timer: Timer::from_seconds(0.2, TimerMode::Once),
                        action: UnitAction::Move(target_tile),
                    }
                } else {
                    *current_action = CurrentAction::default();
                }
            }
        }
    }
}

/// An action that a unit can take.
#[derive(Default)]
pub(super) enum UnitAction {
    #[default]
    Idle,
    Move(TilePos),
}

#[derive(Component, Default)]
/// The action a unit is undertaking.
pub(super) struct CurrentAction {
    /// The type of action being undertaken.
    action: UnitAction,
    /// The amount of time left to complete the action.
    timer: Timer,
}

impl CurrentAction {
    /// Get the action that the unit is currently undertaking.
    pub(super) fn action(&self) -> &UnitAction {
        &self.action
    }
}
