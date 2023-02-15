//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;

use crate::items::ItemId;
use crate::organisms::units::UnitId;
use crate::simulation::geometry::TilePos;

use self::events::MoveThisTurn;

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

/// Events that define what each unit is doing during their turn.
pub(crate) mod events {
    use bevy::{
        ecs::{entity::Entity, system::SystemParam},
        prelude::EventWriter,
    };

    use crate::simulation::geometry::TilePos;

    /// A struct that wraps all of the events defined in this module
    #[derive(SystemParam)]
    pub(crate) struct BehaviorEventWriters<'w, 's> {
        /// Writes [`IdleThisTurn`] events
        #[allow(dead_code)]
        pub(crate) idle_this_turn: EventWriter<'w, 's, IdleThisTurn>,
        /// Writes [`MoveThisTurn`] events
        pub(crate) move_this_turn: EventWriter<'w, 's, MoveThisTurn>,
        /// Writes [`PickUpThisTurn`] events
        #[allow(dead_code)]
        pub(crate) pick_up_this_turn: EventWriter<'w, 's, PickUpThisTurn>,
        /// Writes [`DropOffThisTurn`] events
        #[allow(dead_code)]
        pub(crate) drop_off_this_turn: EventWriter<'w, 's, DropOffThisTurn>,
        /// Writes [`WorkThisTurn`] events
        #[allow(dead_code)]
        pub(crate) work_this_turn: EventWriter<'w, 's, WorkThisTurn>,
    }

    /// The unit in this event is idle this turn.
    pub(crate) struct IdleThisTurn {
        /// The unit performing the action
        #[allow(dead_code)]
        pub(crate) unit_entity: Entity,
    }

    /// The unit in this event is moving to another tile
    pub(crate) struct MoveThisTurn {
        /// The unit performing the action
        #[allow(dead_code)]
        pub(crate) unit_entity: Entity,
        /// The tile to be moved into
        #[allow(dead_code)]
        pub(crate) target_tile: TilePos,
    }

    /// The unit in this event is picking up an object
    pub(crate) struct PickUpThisTurn {
        /// The unit performing the action
        #[allow(dead_code)]
        pub(crate) unit_entity: Entity,
        /// The tile to be moved to
        #[allow(dead_code)]
        pub(crate) pickup_tile: TilePos,
    }

    /// The unit in this event is dropping off an object
    pub(crate) struct DropOffThisTurn {
        /// The unit performing the action
        #[allow(dead_code)]
        pub(crate) unit_entity: Entity,
        /// The tile to be moved to
        #[allow(dead_code)]
        pub(crate) dropoff_tile: TilePos,
    }

    /// The unit in this event is performing work at a structure
    pub(crate) struct WorkThisTurn {
        /// The unit performing the action
        #[allow(dead_code)]
        pub(crate) unit_entity: Entity,
        /// The tile that contains the structure to work at
        #[allow(dead_code)]
        pub(crate) working_at: TilePos,
    }
}

/// Choose this unit's new goal if needed
pub(crate) fn choose_goal(mut units_query: Query<(&UnitId, &mut CurrentGoal)>) {
    for (_unit, current_goal) in units_query.iter_mut() {
        // Check to see if any of the possible goals are high enough priority to swap to
        if *current_goal == CurrentGoal::Wander {
            //todo!()
        }
    }
}

/// Choose the unit's action for this turn
pub(crate) fn choose_action(
    units_query: Query<(Entity, &TilePos, &CurrentGoal), With<UnitId>>,
    mut behavior_event_writer: events::BehaviorEventWriters,
) {
    for (unit_entity, _unit_tile_pos, current_goal) in units_query.iter() {
        if let Some(_required_interactable) = current_goal.required_interactable() {
            // TODO: check neighboring entities
        } else {
            // TODO: actually pick a tile
            let target_tile = TilePos::default();
            behavior_event_writer.move_this_turn.send(MoveThisTurn {
                unit_entity,
                target_tile,
            });
        }
    }
}
