//! What are units currently doing?

use bevy::{ecs::query::WorldQuery, prelude::*};
use core::fmt::Display;
use leafwing_abilities::prelude::Pool;
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng};

use crate::{
    items::{ItemCount, ItemId, ItemManifest},
    organisms::energy::EnergyPool,
    signals::Signals,
    simulation::geometry::{Facing, MapGeometry, RotationDirection, TilePos},
    structures::crafting::{InputInventory, OutputInventory},
};

use super::{goals::Goal, hunger::Diet, item_interaction::HeldItem, UnitId};

/// Ticks the timer for each [`CurrentAction`].
pub(super) fn advance_action_timer(mut units_query: Query<&mut CurrentAction>, time: Res<Time>) {
    let delta = time.delta();

    for mut current_action in units_query.iter_mut() {
        current_action.timer.tick(delta);
    }
}

/// Choose the unit's action for this turn
pub(super) fn choose_actions(
    mut units_query: Query<(&TilePos, &Facing, &Goal, &mut CurrentAction, &HeldItem), With<UnitId>>,
    input_inventory_query: Query<&InputInventory>,
    output_inventory_query: Query<&OutputInventory>,
    map_geometry: Res<MapGeometry>,
    signals: Res<Signals>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (&unit_tile_pos, facing, goal, mut action, held_item) in units_query.iter_mut() {
        if action.finished() {
            *action = match goal {
                // Alternate between spinning and moving forward.
                Goal::Wander => match action.action() {
                    UnitAction::Spin { .. } => {
                        CurrentAction::move_forward(unit_tile_pos, facing, map_geometry)
                    }
                    _ => CurrentAction::random_spin(rng),
                },
                Goal::Pickup(item_id) => {
                    let maybe_item = held_item.item_id();
                    if maybe_item.is_some() && maybe_item.unwrap() != *item_id {
                        CurrentAction::abandon()
                    } else {
                        CurrentAction::find_item(
                            *item_id,
                            unit_tile_pos,
                            facing,
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
                            facing,
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
                            facing,
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

/// Exhaustively handles each planned action
pub(super) fn handle_actions(
    mut unit_query: Query<ActionDataQuery>,
    mut input_query: Query<&mut InputInventory>,
    mut output_query: Query<&mut OutputInventory>,
    map_geometry: Res<MapGeometry>,
    item_manifest: Res<ItemManifest>,
) {
    let item_manifest = &*item_manifest;

    for mut unit in unit_query.iter_mut() {
        if unit.action.finished() {
            match unit.action.action() {
                UnitAction::Idle => (),
                UnitAction::PickUp {
                    item_id,
                    output_entity,
                } => {
                    if let Ok(mut output_inventory) = output_query.get_mut(*output_entity) {
                        // Transfer one item at a time
                        let item_count = ItemCount::new(*item_id, 1);
                        let _transfer_result = output_inventory.transfer_item(
                            &item_count,
                            &mut unit.held_item.inventory,
                            item_manifest,
                        );

                        // If our unit's all loaded, swap to delivering it
                        *unit.goal = if unit.held_item.is_full() {
                            Goal::DropOff(*item_id)
                        // If we can carry more, try and grab more items
                        } else {
                            Goal::Pickup(*item_id)
                        }
                    }
                }
                UnitAction::DropOff {
                    item_id,
                    input_entity,
                } => {
                    if let Ok(mut input_inventory) = input_query.get_mut(*input_entity) {
                        // Transfer one item at a time
                        let item_count = ItemCount::new(*item_id, 1);
                        let _transfer_result = unit.held_item.transfer_item(
                            &item_count,
                            &mut input_inventory.inventory,
                            item_manifest,
                        );

                        // If our unit is unloaded, swap to wandering to find something else to do
                        *unit.goal = if unit.held_item.is_empty() {
                            Goal::Wander
                        // If we still have items, keep unloading
                        } else {
                            Goal::DropOff(*item_id)
                        }
                    }
                }
                UnitAction::Spin { rotation_direction } => match rotation_direction {
                    RotationDirection::Left => unit.facing.rotate_left(),
                    RotationDirection::Right => unit.facing.rotate_right(),
                },
                UnitAction::MoveForward => {
                    let direction = unit.facing.direction;
                    let target_tile = unit.tile_pos.neighbor(direction);

                    *unit.tile_pos = target_tile;
                    unit.transform.translation = target_tile.into_world_pos(&map_geometry);
                }
                UnitAction::Eat => {
                    let item_count = ItemCount::new(unit.diet.item(), 1);
                    let consumption_result = unit.held_item.remove_item_all_or_nothing(&item_count);

                    match consumption_result {
                        Ok(_) => {
                            let proposed = unit.energy_pool.current() + unit.diet.energy();
                            unit.energy_pool.set_current(proposed);
                        }
                        Err(error) => {
                            error!("{error:?}: unit tried to eat the wrong thing!")
                        }
                    }
                }
                UnitAction::Abandon => {
                    // TODO: actually put these dropped items somewhere
                    *unit.held_item = HeldItem::default();
                }
            }
        }
    }
}

/// All of the data needed to handle unit actions correctly
#[derive(WorldQuery)]
#[world_query(mutable)]
pub(super) struct ActionDataQuery {
    /// The unit's goal
    goal: &'static mut Goal,
    /// The unit's action
    action: &'static CurrentAction,
    /// What the unit is holding
    held_item: &'static mut HeldItem,
    /// The unit's spatial position for rendering
    transform: &'static mut Transform,
    /// The tile that the unit is on
    tile_pos: &'static mut TilePos,
    /// What the unit eats
    diet: &'static Diet,
    /// How much energy the unit has
    energy_pool: &'static mut EnergyPool,
    /// The direction this unit is facing
    facing: &'static mut Facing,
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
        /// The direction to turn in.
        rotation_direction: RotationDirection,
    },
    /// Move one tile forward, as determined by the unit's [`Facing`].
    MoveForward,
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
            UnitAction::MoveForward => "Moving forward".to_string(),
            UnitAction::Eat => "Eating".to_string(),
            UnitAction::Abandon => "Abandoning held object".to_string(),
        };

        write!(f, "{string}")
    }
}

#[derive(Component, Clone, Debug)]
/// The action a unit is undertaking.
pub(crate) struct CurrentAction {
    /// The type of action being undertaken.
    action: UnitAction,
    /// The amount of time left to complete the action.
    timer: Timer,
}

impl Default for CurrentAction {
    fn default() -> Self {
        CurrentAction::idle()
    }
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
    #[allow(clippy::too_many_arguments)]
    fn find_item(
        item_id: ItemId,
        unit_tile_pos: TilePos,
        facing: &Facing,
        goal: &Goal,
        output_inventory_query: &Query<&OutputInventory>,
        signals: &Signals,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let neighboring_tiles = unit_tile_pos.all_neighbors(map_geometry);
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
            CurrentAction::move_or_spin(unit_tile_pos, upstream, facing, map_geometry)
        } else {
            CurrentAction::idle()
        }
    }

    /// Attempt to located a place to put an item of type `item_id`.
    #[allow(clippy::too_many_arguments)]
    fn find_receptacle(
        item_id: ItemId,
        unit_tile_pos: TilePos,
        facing: &Facing,
        goal: &Goal,
        input_inventory_query: &Query<&InputInventory>,
        signals: &Signals,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let neighboring_tiles = unit_tile_pos.all_neighbors(map_geometry);
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
            CurrentAction::move_or_spin(unit_tile_pos, upstream, facing, map_geometry)
        } else {
            CurrentAction::idle()
        }
    }

    /// Spins 60 degrees left or right.
    pub(super) fn spin(rotation_direction: RotationDirection) -> Self {
        CurrentAction {
            action: UnitAction::Spin { rotation_direction },
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }

    /// Rotate to face the `required_direction`.
    fn spin_towards(facing: &Facing, required_direction: hexx::Direction) -> Self {
        let mut working_direction_left = facing.direction;
        let mut working_direction_right = facing.direction;

        // Let's race!
        // Left gets an arbitrary unfair advantage though.
        // PERF: this could use a lookup table instead, and would probably be faster
        loop {
            working_direction_left = working_direction_left.left();
            if working_direction_left == required_direction {
                return CurrentAction::spin(RotationDirection::Left);
            }

            working_direction_right = working_direction_right.right();
            if working_direction_right == required_direction {
                return CurrentAction::spin(RotationDirection::Right);
            }
        }
    }

    /// Spins 60 degrees in a random direction
    pub(super) fn random_spin(rng: &mut ThreadRng) -> Self {
        let rotation_direction = RotationDirection::random(rng);

        CurrentAction::spin(rotation_direction)
    }

    /// Move toward the tile this unit is facing if able
    pub(super) fn move_forward(
        unit_tile_pos: TilePos,
        facing: &Facing,
        map_geometry: &MapGeometry,
    ) -> Self {
        let target_tile = unit_tile_pos.neighbor(facing.direction);

        if map_geometry.is_passable(target_tile) {
            CurrentAction {
                action: UnitAction::MoveForward,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            }
        } else {
            CurrentAction::idle()
        }
    }

    /// Attempt to move toward the `target_tile_pos`.
    pub(super) fn move_or_spin(
        unit_tile_pos: TilePos,
        target_tile_pos: TilePos,
        facing: &Facing,
        map_geometry: &MapGeometry,
    ) -> Self {
        let required_direction = unit_tile_pos.direction_to(target_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction::move_forward(unit_tile_pos, facing, map_geometry)
        } else {
            CurrentAction::spin_towards(facing, required_direction)
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
