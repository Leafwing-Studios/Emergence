//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
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
pub(crate) enum Goal {
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

impl Goal {
    /// Choose an action based on the goal and the information about the environment.
    fn choose_action(
        &self,
        unit_tile_pos: TilePos,
        map_geometry: &MapGeometry,
        rng: &mut ThreadRng,
    ) -> CurrentAction {
        match self {
            Goal::Wander => {
                if let Some(random_neighbor) =
                    unit_tile_pos.choose_random_empty_neighbor(rng, map_geometry)
                {
                    CurrentAction::move_to(random_neighbor)
                } else {
                    CurrentAction::idle()
                }
            }
            Goal::Pickup(_) => todo!(),
            Goal::DropOff(_) => todo!(),
            Goal::Work(_) => todo!(),
        }
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(mut units_query: Query<&mut Goal>) {
    // TODO: pick goal intelligently based on local environment
    let possible_goals = vec![Goal::Wander];
    let rng = &mut thread_rng();

    for mut goal in units_query.iter_mut() {
        *goal = possible_goals.choose(rng).unwrap().clone();
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
pub(super) fn choose_actions(
    mut units_query: Query<(&TilePos, &Goal, &mut CurrentAction), With<UnitId>>,
    map_geometry: Res<MapGeometry>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (&unit_tile_pos, current_goal, mut current_action) in units_query.iter_mut() {
        if current_action.finished() {
            *current_action = current_goal.choose_action(unit_tile_pos, map_geometry, rng);
        }
    }
}

/// An action that a unit can take.
#[derive(Default)]
pub(super) enum UnitAction {
    /// Do nothing for now
    #[default]
    Idle,
    /// Move to the tile position
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

    /// Have we waited long enough to perform this action?
    pub(super) fn finished(&self) -> bool {
        self.timer.finished()
    }

    /// Move to the adjacent tile
    fn move_to(target_tile: TilePos) -> Self {
        CurrentAction {
            action: UnitAction::Move(target_tile),
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }

    /// Wait, as there is nothing to be done.
    fn idle() -> Self {
        CurrentAction {
            action: UnitAction::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
