//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::interactable::{self, Interactable};
use crate::organisms::units::Unit;

/// A unit's current goals.
///
/// Units will be fully concentrated on any task other than [`CurrentGoal::Wander`] until it is complete (or overridden).
///
/// This component serves as a state machine.
#[derive(Component, PartialEq, Eq, Clone, Default)]
pub enum CurrentGoal {
    /// Attempting to find something useful to do
    ///
    /// Units will try and follow a signal, if they can pick up a trail, but will not fixate on it until the signal is strong enough.
    #[default]
    Wander,
    /// Attempting to pick up an object
    Pickup(Interactable),
    /// Attempting to drop off an object
    DropOff(Interactable),
    /// Attempting to perform work at a structure
    Work(Interactable),
}

impl CurrentGoal {
    fn required_interactable(&self) -> Option<Interactable> {
        match self {
            CurrentGoal::Wander => None,
            CurrentGoal::Pickup(interactable) => Some(interactable.clone()),
            CurrentGoal::DropOff(interactable) => Some(interactable.clone()),
            CurrentGoal::Work(interactable) => Some(interactable.clone()),
        }
    }
}

/// Events that define what each unit is doing during their turn.
pub mod events {
    use bevy::{
        ecs::{entity::Entity, system::SystemParam},
        prelude::EventWriter,
    };
    use bevy_ecs_tilemap::tiles::TilePos;

    /// A struct that wraps all of the events defined in this module
    #[derive(SystemParam)]
    pub struct BehaviorEventWriters<'w, 's> {
        pub idle_this_turn: EventWriter<'w, 's, IdleThisTurn>,
        pub move_this_turn: EventWriter<'w, 's, MoveThisTurn>,
        pub pick_up_this_turn: EventWriter<'w, 's, PickUpThisTurn>,
        pub drop_off_this_turn: EventWriter<'w, 's, DropOffThisTurn>,
        pub work_this_turn: EventWriter<'w, 's, WorkThisTurn>,
    }

    /// The unit in this event is idle this turn.
    pub struct IdleThisTurn {
        /// The unit performing the action
        pub unit: Entity,
    }

    /// The unit in this event is moving to another tile
    pub struct MoveThisTurn {
        /// The unit performing the action
        pub unit: Entity,
        /// The tile to be moved into
        pub target: TilePos,
    }

    /// The unit in this event is picking up an object
    pub struct PickUpThisTurn {
        /// The unit performing the action
        pub unit: Entity,
        /// The tile to be moved to
        pub pickup_tile: TilePos,
    }

    /// The unit in this event is dropping off an object
    pub struct DropOffThisTurn {
        /// The unit performing the action
        pub unit: Entity,
        /// The tile to be moved to
        pub dropoff_tile: TilePos,
    }

    /// The unit in this event is performing work at a structure
    pub struct WorkThisTurn {
        /// The unit performing the action
        pub unit: Entity,
        /// The tile that contains the structure to work at
        pub working_at: TilePos,
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(mut units_query: Query<(&Unit, &mut CurrentGoal)>) {
    for (unit, current_goal) in units_query.iter_mut() {
        // Check to see if any of the possible goals are high enough priority to swap to
        if *current_goal == CurrentGoal::Wander {
            todo!()
        }
    }
}

/// Choose the unit's action for this turn
pub(super) fn choose_action(
    units_query: Query<(Entity, &TilePos, &CurrentGoal), With<Unit>>,
    interactables_query: Query<(Entity, &TilePos, &Interactable)>,
    behavior_event_writer: events::BehaviorEventWriters,
) {
    for (unit_entity, unit_tile_pos, current_goal) in units_query.iter() {
        if let Some(required_interactable) = current_goal.required_interactable() {
            // TODO: use HexNeighbors methods to find appropriate neighboring entities
        }
    }
}
