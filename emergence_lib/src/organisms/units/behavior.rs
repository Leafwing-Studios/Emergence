//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;
use core::fmt::Display;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::items::ItemId;
use crate::organisms::units::UnitId;
use crate::simulation::geometry::{MapGeometry, TilePos};
use crate::structures::crafting::{InputInventory, OutputInventory};
use crate::structures::StructureId;

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
    Pickup(ItemId),
    /// Attempting to drop off an object
    #[allow(dead_code)]
    DropOff(ItemId),
    /// Attempting to perform work at a structure
    #[allow(dead_code)]
    Work(StructureId),
}

impl Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match self {
            Goal::Wander => "Wander".to_string(),
            Goal::Pickup(item) => format!("Pickup {item}"),
            Goal::DropOff(item) => format!("Dropoff {item}"),
            Goal::Work(structure) => format!("Work at {structure}"),
        };

        write!(f, "{string}")
    }
}

impl Goal {
    /// Choose an action based on the goal and the information about the environment.
    fn choose_action(
        &self,
        unit_tile_pos: TilePos,
        map_geometry: &MapGeometry,
        input_inventory_query: &Query<&InputInventory>,
        output_inventory_query: &Query<&OutputInventory>,
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
            Goal::Pickup(item_id) => {
                let neighboring_tiles = unit_tile_pos.neighbors(map_geometry);
                let mut entities_with_desired_item: Vec<Entity> = Vec::new();

                for tile_pos in neighboring_tiles {
                    if let Some(&structure_entity) = map_geometry.structure_index.get(&tile_pos) {
                        if let Ok(output_inventory) = output_inventory_query.get(structure_entity) {
                            if output_inventory.item_count(item_id) > 0 {
                                entities_with_desired_item.push(structure_entity);
                            }
                        }
                    }
                }

                if let Some(output_entity) = entities_with_desired_item.choose(rng) {
                    CurrentAction::pickup(item_id.clone(), *output_entity)
                } else {
                    // TODO: walk towards destination more intelligently
                    Goal::Wander.choose_action(
                        unit_tile_pos,
                        map_geometry,
                        input_inventory_query,
                        output_inventory_query,
                        rng,
                    )
                }
            }
            Goal::DropOff(item_id) => {
                let neighboring_tiles = unit_tile_pos.neighbors(map_geometry);
                let mut entities_with_desired_item: Vec<Entity> = Vec::new();

                for tile_pos in neighboring_tiles {
                    if let Some(&structure_entity) = map_geometry.structure_index.get(&tile_pos) {
                        if let Ok(input_inventory) = input_inventory_query.get(structure_entity) {
                            if input_inventory.remaining_reserved_space_for_item(item_id) > 0 {
                                entities_with_desired_item.push(structure_entity);
                            }
                        }
                    }
                }

                if let Some(input_entity) = entities_with_desired_item.choose(rng) {
                    CurrentAction::dropoff(item_id.clone(), *input_entity)
                } else {
                    // TODO: walk towards destination more intelligently
                    Goal::Wander.choose_action(
                        unit_tile_pos,
                        map_geometry,
                        input_inventory_query,
                        output_inventory_query,
                        rng,
                    )
                }
            }
            Goal::Work(_) => todo!(),
        }
    }
}

/// Choose this unit's new goal if needed
pub(super) fn choose_goal(mut units_query: Query<&mut Goal>) {
    // TODO: pick goal intelligently based on local environment
    let possible_goals = vec![Goal::Wander, Goal::Pickup(ItemId::acacia_leaf())];
    let rng = &mut thread_rng();

    for mut goal in units_query.iter_mut() {
        // By default, goals are reset to wandering when completed.
        // Pick a new goal when wandering.
        if let Goal::Wander = *goal {
            *goal = possible_goals.choose(rng).unwrap().clone();
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
pub(super) fn choose_actions(
    mut units_query: Query<(&TilePos, &Goal, &mut CurrentAction), With<UnitId>>,
    input_inventory_query: Query<&InputInventory>,
    output_inventory_query: Query<&OutputInventory>,
    map_geometry: Res<MapGeometry>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (&unit_tile_pos, current_goal, mut current_action) in units_query.iter_mut() {
        if current_action.finished() {
            *current_action = current_goal.choose_action(
                unit_tile_pos,
                map_geometry,
                &input_inventory_query,
                &output_inventory_query,
                rng,
            );
        }
    }
}

/// An action that a unit can take.
#[derive(Default, Clone, Debug)]
pub(super) enum UnitAction {
    /// Do nothing for now
    #[default]
    Idle,
    /// Pick up the `item_id` from the `output_entity.
    PickUp {
        /// The item to pickup.
        item_id: ItemId,
        /// The entity to grab it from, which must have an [`OutputInventory`] component.
        output_entity: Entity,
    },
    /// Drops off the `item_id` at the `output_entity.
    DropOff {
        /// The item that this unit is carrying that we should drop off.
        item_id: ItemId,
        /// The entity to drop it off at, which must have an [`InputInventory`] component.
        input_entity: Entity,
    },
    /// Move to the tile position
    Move(TilePos),
}

impl Display for UnitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match self {
            UnitAction::Idle => "Idling".to_string(),
            UnitAction::PickUp {
                item_id,
                output_entity,
            } => format!("Picking up {item_id} from {output_entity:?}"),
            UnitAction::DropOff {
                item_id,
                input_entity,
            } => format!("Dropping off {item_id} at {input_entity:?}"),
            UnitAction::Move(tile_pos) => format!("Moving to {tile_pos}"),
        };

        write!(f, "{string}")
    }
}

#[derive(Component, Clone, Default, Debug)]
/// The action a unit is undertaking.
pub(crate) struct CurrentAction {
    /// The type of action being undertaken.
    action: UnitAction,
    /// The amount of time left to complete the action.
    timer: Timer,
}

impl Display for CurrentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = &self.action;
        let time_remaining = self.timer.remaining_secs();

        write!(f, "{action} for the next {time_remaining:.2} s.")
    }
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
    pub(super) fn move_to(target_tile: TilePos) -> Self {
        CurrentAction {
            action: UnitAction::Move(target_tile),
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }

    /// Wait, as there is nothing to be done.
    pub(super) fn idle() -> Self {
        CurrentAction {
            action: UnitAction::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }

    /// Picks up the `item_id` at the `output_entity`.
    pub(super) fn pickup(item_id: ItemId, output_entity: Entity) -> Self {
        CurrentAction {
            action: UnitAction::PickUp {
                item_id,
                output_entity,
            },
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }

    /// Drops off the `item_id` at the `input_entity`.
    pub(super) fn dropoff(item_id: ItemId, input_entity: Entity) -> Self {
        CurrentAction {
            action: UnitAction::DropOff {
                item_id,
                input_entity,
            },
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
