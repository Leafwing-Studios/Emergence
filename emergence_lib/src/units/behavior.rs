//! What are units doing, and why?
//!
//! The AI model of Emergence.

use bevy::prelude::*;
use core::fmt::Display;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::items::ItemId;
use crate::signals::{SignalType, Signals};
use crate::simulation::geometry::{Facing, MapGeometry, RotationDirection, TilePos};
use crate::structures::crafting::{InputInventory, OutputInventory};
use crate::structures::StructureId;
use crate::units::UnitId;

use super::item_interaction::HeldItem;

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
    /// Attempt to feed self
    Eat(ItemId),
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
pub(super) fn choose_goal(mut units_query: Query<(&TilePos, &mut Goal)>, signals: Res<Signals>) {
    let rng = &mut thread_rng();

    for (&tile_pos, mut goal) in units_query.iter_mut() {
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
                }
            }
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
    mut units_query: Query<(&TilePos, &Goal, &mut CurrentAction, &HeldItem), With<UnitId>>,
    input_inventory_query: Query<&InputInventory>,
    output_inventory_query: Query<&OutputInventory>,
    map_geometry: Res<MapGeometry>,
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (&unit_tile_pos, goal, mut current_action, held_item) in units_query.iter_mut() {
        if current_action.finished() {
            *current_action = match goal {
                Goal::Wander => CurrentAction::wander(unit_tile_pos, rng, map_geometry),
                Goal::Pickup(item_id) => {
                    let maybe_item = held_item.item_id();
                    if maybe_item.is_some() && maybe_item.unwrap() != *item_id {
                        CurrentAction::abandon()
                    } else {
                        CurrentAction::find_item(
                            *item_id,
                            unit_tile_pos,
                            goal,
                            &output_inventory_query,
                            &signals,
                            rng,
                            map_geometry,
                        )
                    }
                }
                Goal::DropOff(item_id) => {
                    let maybe_item = held_item.item_id();
                    if maybe_item.is_some() && maybe_item.unwrap() != *item_id {
                        CurrentAction::abandon()
                    } else {
                        CurrentAction::find_receptacle(
                            *item_id,
                            unit_tile_pos,
                            goal,
                            &input_inventory_query,
                            &signals,
                            rng,
                            map_geometry,
                        )
                    }
                }
                Goal::Eat(item_id) => {
                    if let Some(held_item) = held_item.item_id() {
                        if held_item == *item_id {
                            CurrentAction::eat()
                        } else {
                            CurrentAction::abandon()
                        }
                    } else {
                        CurrentAction::find_item(
                            *item_id,
                            unit_tile_pos,
                            goal,
                            &output_inventory_query,
                            &signals,
                            rng,
                            map_geometry,
                        )
                    }
                }
                Goal::Work(_) => todo!(),
            }
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
    /// Spin left or right.
    Spin {
        rotation_direction: RotationDirection,
    },
    /// Move to the tile position
    Move(TilePos),
    /// Eats one of the currently held object
    Eat,
    /// Abandon whatever you are currently holding
    Abandon,
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
            UnitAction::Spin { rotation_direction } => format!("Spinning {rotation_direction}"),
            UnitAction::Move(tile_pos) => format!("Moving to {tile_pos}"),
            UnitAction::Eat => "Eating".to_string(),
            UnitAction::Abandon => "Abandoning held object.".to_string(),
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

        write!(f, "{action}\nRemaining: {time_remaining:.2} s.")
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

    /// Attempt to locate a source of the provided `item_id`.
    fn find_item(
        item_id: ItemId,
        unit_tile_pos: TilePos,
        goal: &Goal,
        output_inventory_query: &Query<&OutputInventory>,
        signals: &Signals,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
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
            CurrentAction::pickup(item_id, *output_entity)
        } else if let Some(upstream) = signals.upstream(unit_tile_pos, goal, map_geometry) {
            CurrentAction::move_to(upstream)
        } else {
            CurrentAction::wander(unit_tile_pos, rng, map_geometry)
        }
    }

    /// Attempt to located a place to put an item of type `item_id`.
    fn find_receptacle(
        item_id: ItemId,
        unit_tile_pos: TilePos,
        goal: &Goal,
        input_inventory_query: &Query<&InputInventory>,
        signals: &Signals,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let neighboring_tiles = unit_tile_pos.neighbors(map_geometry);
        let mut entities_with_desired_item: Vec<Entity> = Vec::new();

        for tile_pos in neighboring_tiles {
            // Ghosts
            if let Some(&ghost_entity) = map_geometry.ghost_index.get(&tile_pos) {
                if let Ok(input_inventory) = input_inventory_query.get(ghost_entity) {
                    if input_inventory.remaining_reserved_space_for_item(item_id) > 0 {
                        entities_with_desired_item.push(ghost_entity);
                    }
                }
            }

            // Structures
            if let Some(&structure_entity) = map_geometry.structure_index.get(&tile_pos) {
                if let Ok(input_inventory) = input_inventory_query.get(structure_entity) {
                    if input_inventory.remaining_reserved_space_for_item(item_id) > 0 {
                        entities_with_desired_item.push(structure_entity);
                    }
                }
            }
        }

        if let Some(input_entity) = entities_with_desired_item.choose(rng) {
            CurrentAction::dropoff(item_id, *input_entity)
        } else if let Some(upstream) = signals.upstream(unit_tile_pos, goal, map_geometry) {
            CurrentAction::move_to(upstream)
        } else {
            CurrentAction::wander(unit_tile_pos, rng, map_geometry)
        }
    }

    /// Wander to an adjacent tile, chosen randomly
    fn wander(
        unit_tile_pos: TilePos,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        if let Some(random_neighbor) = unit_tile_pos.choose_random_empty_neighbor(rng, map_geometry)
        {
            CurrentAction::move_to(random_neighbor)
        } else {
            CurrentAction::idle()
        }
    }

    /// Spin to face the selected direction
    pub(super) fn spin_towards(current_facing: Facing, desired_facing: Facing) -> Self {
        assert!(current_facing != desired_facing);

        let mut rotations_needed = 0;
        let mut working_direction = desired_facing.direction;

        while working_direction != current_facing.direction {
            working_direction = working_direction.left();
            rotations_needed += 1;
        }

        let rotation_direction = if rotations_needed <= 3 {
            RotationDirection::Left
        } else {
            RotationDirection::Right
        };

        CurrentAction {
            action: UnitAction::Spin { rotation_direction },
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }

    /// Move to the adjacent tile
    pub(super) fn move_to(target_tile: TilePos) -> Self {
        CurrentAction {
            action: UnitAction::Move(target_tile),
            timer: Timer::from_seconds(0.5, TimerMode::Once),
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
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }

    /// Drops off the `item_id` at the `input_entity`.
    pub(super) fn dropoff(item_id: ItemId, input_entity: Entity) -> Self {
        CurrentAction {
            action: UnitAction::DropOff {
                item_id,
                input_entity,
            },
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }

    /// Eats one of the currently held item.
    pub(super) fn eat() -> Self {
        CurrentAction {
            action: UnitAction::Eat,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }

    /// Eats one of the currently held item.
    pub(super) fn abandon() -> Self {
        CurrentAction {
            action: UnitAction::Abandon,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
