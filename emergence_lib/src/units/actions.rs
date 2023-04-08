//! What are units currently doing?

use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};
use leafwing_abilities::prelude::Pool;
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng};

use crate::{
    asset_management::manifest::Id,
    construction::{
        demolition::{DemolitionQuery, MarkedForDemolition},
        ghosts::WorkplaceId,
        terraform::TerraformingAction,
    },
    crafting::{
        components::{
            AddToInputError, CraftingState, InputInventory, OutputInventory, StorageInventory,
            WorkersPresent,
        },
        item_tags::ItemKind,
    },
    items::{errors::AddOneItemError, item_manifest::ItemManifest, ItemCount},
    organisms::{energy::EnergyPool, lifecycle::Lifecycle},
    signals::{SignalType, Signals},
    simulation::geometry::{Facing, Height, MapGeometry, RotationDirection, TilePos},
    structures::{commands::StructureCommandsExt, structure_manifest::Structure},
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

use super::{
    goals::Goal,
    impatience::ImpatiencePool,
    item_interaction::UnitInventory,
    unit_manifest::{Unit, UnitManifest},
};

/// Ticks the timer for each [`CurrentAction`].
pub(super) fn advance_action_timer(
    mut units_query: Query<&mut CurrentAction>,
    time: Res<FixedTime>,
) {
    let delta = time.period;

    for mut current_action in units_query.iter_mut() {
        current_action.timer.tick(delta);
    }
}

/// Choose the unit's action for this turn
pub(super) fn choose_actions(
    mut units_query: Query<
        (&TilePos, &Facing, &Goal, &mut CurrentAction, &UnitInventory),
        With<Id<Unit>>,
    >,
    // We shouldn't be dropping off new stuff at structures that are about to be destroyed!
    input_inventory_query: Query<&InputInventory, Without<MarkedForDemolition>>,
    // But we can take their items away
    output_inventory_query: Query<&OutputInventory>,
    storage_inventory_query: Query<&StorageInventory>,
    workplace_query: WorkplaceQuery,
    demolition_query: DemolitionQuery,
    map_geometry: Res<MapGeometry>,
    signals: Res<Signals>,
    terrain_query: Query<&Id<Terrain>>,
    terrain_storage_query: Query<&StorageInventory, With<Id<Terrain>>>,
    terrain_manifest: Res<TerrainManifest>,
    item_manifest: Res<ItemManifest>,
) {
    let rng = &mut thread_rng();
    let map_geometry = map_geometry.into_inner();

    for (&unit_tile_pos, facing, goal, mut current_action, unit_inventory) in units_query.iter_mut()
    {
        if current_action.finished() {
            let previous_action = current_action.action.clone();

            *current_action = match goal {
                // Alternate between spinning and moving forward.
                Goal::Wander { .. } => CurrentAction::wander(
                    previous_action,
                    unit_tile_pos,
                    facing,
                    map_geometry,
                    &terrain_query,
                    &terrain_manifest,
                    rng,
                ),
                Goal::Fetch(item_kind)
                | Goal::Deliver(item_kind)
                | Goal::Store(item_kind)
                | Goal::Remove(item_kind) => {
                    // If we're holding the wrong thing, drop it.
                    if unit_inventory.is_some()
                        && !item_kind.matches(unit_inventory.unwrap(), &item_manifest)
                    {
                        CurrentAction::abandon(
                            previous_action,
                            unit_tile_pos,
                            unit_inventory,
                            map_geometry,
                            &item_manifest,
                            &terrain_storage_query,
                            &terrain_manifest,
                            &terrain_query,
                            facing,
                            rng,
                        )
                    } else {
                        CurrentAction::find_place_for_item(
                            *item_kind,
                            goal.delivery_mode().unwrap(),
                            goal.purpose(),
                            unit_tile_pos,
                            facing,
                            goal,
                            &input_inventory_query,
                            &output_inventory_query,
                            &storage_inventory_query,
                            &signals,
                            rng,
                            &item_manifest,
                            &terrain_query,
                            &terrain_manifest,
                            map_geometry,
                        )
                    }
                }
                Goal::Eat(item_kind) => {
                    if let Some(held_item) = unit_inventory.held_item {
                        if item_kind.matches(held_item, &item_manifest) {
                            CurrentAction::eat()
                        } else {
                            CurrentAction::abandon(
                                previous_action,
                                unit_tile_pos,
                                unit_inventory,
                                map_geometry,
                                &item_manifest,
                                &terrain_storage_query,
                                &terrain_manifest,
                                &terrain_query,
                                facing,
                                rng,
                            )
                        }
                    } else {
                        CurrentAction::find_place_for_item(
                            *item_kind,
                            DeliveryMode::PickUp,
                            Purpose::Instrumental,
                            unit_tile_pos,
                            facing,
                            goal,
                            &input_inventory_query,
                            &output_inventory_query,
                            &storage_inventory_query,
                            &signals,
                            rng,
                            &item_manifest,
                            &terrain_query,
                            &terrain_manifest,
                            map_geometry,
                        )
                    }
                }
                Goal::Work(structure_id) => CurrentAction::find_workplace(
                    *structure_id,
                    unit_tile_pos,
                    facing,
                    &workplace_query,
                    &signals,
                    rng,
                    &terrain_query,
                    &terrain_manifest,
                    &item_manifest,
                    map_geometry,
                ),
                Goal::Demolish(structure_id) => CurrentAction::find_demolition_site(
                    *structure_id,
                    unit_tile_pos,
                    facing,
                    &demolition_query,
                    &signals,
                    rng,
                    &item_manifest,
                    &terrain_query,
                    &terrain_manifest,
                    map_geometry,
                ),
            }
        }
    }
}

/// Exhaustively handles the setup for each planned action
pub(super) fn start_actions(
    mut unit_query: Query<&mut CurrentAction>,
    mut workplace_query: Query<&mut WorkersPresent>,
) {
    for mut action in unit_query.iter_mut() {
        if action.just_started {
            if let Some(workplace_entity) = action.action().workplace() {
                if let Ok(mut workers_present) = workplace_query.get_mut(workplace_entity) {
                    // This has a side effect of adding the worker to the workplace
                    let result = workers_present.add_worker();
                    if result.is_err() {
                        *action = CurrentAction::idle();
                    }
                } else {
                    warn!("Unit tried to start working at an entity that is not a workplace!");
                }
            }

            action.just_started = false;
        }
    }
}

/// Exhaustively handles the cleanup for each planned action
pub(super) fn finish_actions(
    mut unit_query: Query<ActionDataQuery>,
    mut inventory_query: Query<
        AnyOf<(
            &mut InputInventory,
            &mut OutputInventory,
            &mut StorageInventory,
        )>,
    >,
    mut workplace_query: Query<(&CraftingState, &mut WorkersPresent)>,
    // This must be compatible with unit_query
    structure_query: Query<&TilePos, (With<Id<Structure>>, Without<Goal>)>,
    map_geometry: Res<MapGeometry>,
    item_manifest: Res<ItemManifest>,
    unit_manifest: Res<UnitManifest>,
    signals: Res<Signals>,
    mut commands: Commands,
) {
    let item_manifest = &*item_manifest;

    for mut unit in unit_query.iter_mut() {
        if unit.action.finished() {
            // Take workers off of the job once actions complete
            if let Some(workplace_entity) = unit.action.action().workplace() {
                if let Ok(workplace) = workplace_query.get_mut(workplace_entity) {
                    let (.., mut workers_present) = workplace;
                    // FIXME: this isn't robust to units dying
                    workers_present.remove_worker();
                }
            }

            match unit.action.action() {
                UnitAction::Idle => {
                    unit.impatience.increment();
                }
                UnitAction::PickUp {
                    item_kind,
                    output_entity,
                } => {
                    if let Ok((_, mut maybe_output_inventory, mut maybe_storage_inventory)) =
                        inventory_query.get_mut(*output_entity)
                    {
                        *unit.goal = match unit.unit_inventory.held_item {
                            // We shouldn't be holding anything yet, but if we are get rid of it
                            Some(held_item_id) => Goal::Store(ItemKind::Single(held_item_id)),
                            None => {
                                let maybe_item_id = if let Some(ref output_inventory) =
                                    maybe_output_inventory
                                {
                                    output_inventory.matching_item_id(*item_kind, item_manifest)
                                } else if let Some(ref storage_inventory) = maybe_storage_inventory
                                {
                                    storage_inventory.matching_item_id(*item_kind, item_manifest)
                                } else {
                                    // The entity must have either an output or storage inventory
                                    unreachable!()
                                };

                                if let Some(item_id) = maybe_item_id {
                                    let item_count = ItemCount::new(item_id, 1);

                                    let transfer_result = match (
                                        &mut maybe_output_inventory,
                                        &mut maybe_storage_inventory,
                                    ) {
                                        (Some(ref mut output_inventory), _) => {
                                            output_inventory.remove_item_all_or_nothing(&item_count)
                                        }
                                        (_, Some(ref mut storage_inventory)) => storage_inventory
                                            .remove_item_all_or_nothing(&item_count),
                                        // The entity must have either an output or storage inventory
                                        _ => unreachable!(),
                                    };

                                    // If our unit's all loaded, swap to delivering it
                                    match transfer_result {
                                        Ok(()) => {
                                            unit.unit_inventory.held_item = Some(item_id);
                                            if signals.detectable(
                                                SignalType::item_signal_types(
                                                    *item_kind,
                                                    item_manifest,
                                                    DeliveryMode::DropOff,
                                                    Purpose::Instrumental,
                                                ),
                                                *unit.tile_pos,
                                            ) {
                                                // If we can see any `Pull` signals of the right type, deliver the item.
                                                Goal::Deliver(*item_kind)
                                            } else {
                                                // Otherwise, simply store it
                                                Goal::Store(*item_kind)
                                            }
                                        }
                                        Err(..) => Goal::Fetch(*item_kind),
                                    }
                                } else {
                                    unit.impatience.increment();
                                    Goal::Fetch(*item_kind)
                                }
                            }
                        }
                    } else {
                        // If the target isn't there, pick a new goal
                        *unit.goal = Goal::default();
                    }
                }
                UnitAction::DropOff {
                    item_kind,
                    input_entity,
                } => {
                    if let Ok((maybe_input_inventory, _, maybe_storage_inventory)) =
                        inventory_query.get_mut(*input_entity)
                    {
                        *unit.goal = match unit.unit_inventory.held_item {
                            // We should be holding something, if we're not find something else to do
                            None => Goal::default(),
                            Some(held_item_id) => {
                                if item_kind.matches(held_item_id, item_manifest) {
                                    let item_count = ItemCount::new(held_item_id, 1);
                                    let transfer_result = if let Some(mut input_inventory) =
                                        maybe_input_inventory
                                    {
                                        input_inventory.fill_with_items(&item_count, item_manifest)
                                    } else if let Some(mut storage_inventory) =
                                        maybe_storage_inventory
                                    {
                                        let storage_result = storage_inventory
                                            .add_item_all_or_nothing(&item_count, item_manifest);
                                        match storage_result {
                                            Ok(()) => Ok(()),
                                            Err(AddOneItemError { excess_count }) => {
                                                Err(AddToInputError::NotEnoughSpace {
                                                    excess_count,
                                                })
                                            }
                                        }
                                    } else {
                                        unreachable!()
                                    };

                                    // If our unit is unloaded, swap to wandering to find something else to do
                                    match transfer_result {
                                        Ok(()) => {
                                            unit.unit_inventory.held_item = None;
                                            Goal::default()
                                        }
                                        Err(..) => {
                                            unit.impatience.increment();
                                            Goal::Store(ItemKind::Single(held_item_id))
                                        }
                                    }
                                } else {
                                    // Somehow we're holding the wrong thing
                                    Goal::Store(ItemKind::Single(held_item_id))
                                }
                            }
                        }
                    } else {
                        // If the target isn't there, pick a new goal
                        *unit.goal = Goal::default();
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
                    unit.transform.translation = target_tile.top_of_tile(&map_geometry);
                }
                UnitAction::Work { structure_entity } => {
                    let mut success = false;

                    if let Ok((CraftingState::InProgress { .. }, workers_present)) =
                        workplace_query.get_mut(*structure_entity)
                    {
                        if workers_present.needs_more() {
                            success = true;
                        }
                    }

                    if !success {
                        *unit.goal = Goal::default();
                    }
                }
                UnitAction::Demolish { structure_entity } => {
                    if let Ok(&structure_tile_pos) = structure_query.get(*structure_entity) {
                        // TODO: this should probably take time and use work?
                        commands.despawn_structure(structure_tile_pos);
                    }

                    // Whether we succeeded or failed, pick something else to do
                    *unit.goal = Goal::default();
                }
                UnitAction::Eat => {
                    if let Some(held_item) = unit.unit_inventory.held_item {
                        let unit_data = unit_manifest.get(*unit.unit_id);

                        let diet = &unit_data.diet;

                        if diet.item_kind().matches(held_item, item_manifest) {
                            unit.unit_inventory.held_item = None;

                            let proposed = unit.energy_pool.current() + diet.energy();
                            unit.energy_pool.set_current(proposed);
                            unit.lifecycle.record_energy_gained(diet.energy());
                        }
                    }
                }
                UnitAction::Abandon => {
                    let terrain_entity = map_geometry.get_terrain(*unit.tile_pos).unwrap();

                    let (_input, _output, maybe_storage) =
                        inventory_query.get_mut(terrain_entity).unwrap();

                    let mut terrain_storage_inventory = maybe_storage.unwrap();

                    if let Some(held_item) = unit.unit_inventory.held_item {
                        let item_count = ItemCount::new(held_item, 1);
                        // Try to transfer the item to the terrain storage
                        if terrain_storage_inventory
                            .add_item_all_or_nothing(&item_count, item_manifest)
                            .is_ok()
                        {
                            unit.unit_inventory.held_item = None;
                        }
                    }
                }
            }
        }
    }
}

/// All of the data needed to handle unit actions correctly
#[derive(WorldQuery)]
#[world_query(mutable)]
pub(super) struct ActionDataQuery {
    /// The [`Id`] of the unit type
    unit_id: &'static Id<Unit>,
    /// The unit's goal
    goal: &'static mut Goal,
    /// The unit's action
    action: &'static CurrentAction,
    /// The unit's progress towards any transformations
    lifecycle: &'static mut Lifecycle,
    /// What the unit is holding
    unit_inventory: &'static mut UnitInventory,
    /// The unit's spatial position for rendering
    transform: &'static mut Transform,
    /// The tile that the unit is on
    tile_pos: &'static mut TilePos,
    /// How much energy the unit has
    energy_pool: &'static mut EnergyPool,
    /// How frustrated this unit is about not being able to progress towards its goal
    impatience: &'static mut ImpatiencePool,
    /// The direction this unit is facing
    facing: &'static mut Facing,
}

/// An action that a unit can take.
#[derive(Default, Clone, Debug)]
pub(super) enum UnitAction {
    /// Do nothing for now
    #[default]
    Idle,
    /// Pick up an item that matches `item_kind` from the `output_entity.
    PickUp {
        /// The item to pickup.
        item_kind: ItemKind,
        /// The entity to grab it from, which must have an [`OutputInventory`] or [`StorageInventory`] component.
        output_entity: Entity,
    },
    /// Drops off an item that matches `item_kind` at the `output_entity`.
    DropOff {
        /// The item that this unit is carrying that we should drop off.
        item_kind: ItemKind,
        /// The entity to drop it off at, which must have an [`InputInventory`] or [`StorageInventory`] component.
        input_entity: Entity,
    },
    /// Perform work at the provided `structure_entity`
    Work {
        /// The structure to work at.
        structure_entity: Entity,
    },
    /// Attempt to deconstruct the provided `structure_entity`
    Demolish {
        /// The structure to work at.
        structure_entity: Entity,
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
    /// Abandon whatever you are currently holding, dropping it on the ground
    Abandon,
}

impl UnitAction {
    /// Gets the workplace [`Entity`] that this action is targeting, if any.
    fn workplace(&self) -> Option<Entity> {
        match self {
            UnitAction::Work { structure_entity }
            | UnitAction::Demolish { structure_entity }
            | UnitAction::DropOff {
                item_kind: _,
                input_entity: structure_entity,
            }
            | UnitAction::PickUp {
                item_kind: _,
                output_entity: structure_entity,
            } => Some(*structure_entity),
            _ => None,
        }
    }

    /// Pretty formatting for this type
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        match self {
            UnitAction::Idle => "Idling".to_string(),
            UnitAction::PickUp {
                item_kind,
                output_entity,
            } => format!(
                "Picking up {} from {output_entity:?}",
                item_manifest.name_of_kind(*item_kind)
            ),
            UnitAction::DropOff {
                item_kind,
                input_entity,
            } => format!(
                "Dropping off {} at {input_entity:?}",
                item_manifest.name_of_kind(*item_kind)
            ),
            UnitAction::Work { structure_entity } => format!("Working at {structure_entity:?}"),
            UnitAction::Demolish { structure_entity } => {
                format!("Demolishing {structure_entity:?}")
            }
            UnitAction::Spin { rotation_direction } => format!("Spinning {rotation_direction}"),
            UnitAction::MoveForward => "Moving forward".to_string(),
            UnitAction::Eat => "Eating".to_string(),
            UnitAction::Abandon => "Abandoning held object".to_string(),
        }
    }
}

#[derive(Component, Clone, Debug)]
/// The action a unit is undertaking.
pub(crate) struct CurrentAction {
    /// The type of action being undertaken.
    action: UnitAction,
    /// The amount of time left to complete the action.
    timer: Timer,
    /// Did this action just start?
    just_started: bool,
}

impl Default for CurrentAction {
    fn default() -> Self {
        CurrentAction::idle()
    }
}

impl CurrentAction {
    /// Pretty formatting for this type
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        let action = &self.action;
        let time_remaining = self.timer.remaining_secs();

        format!(
            "{}\nRemaining: {time_remaining:.2} s.",
            action.display(item_manifest)
        )
    }

    /// Get the action that the unit is currently undertaking.
    pub(super) fn action(&self) -> &UnitAction {
        &self.action
    }

    /// Have we waited long enough to perform this action?
    pub(super) fn finished(&self) -> bool {
        self.timer.finished()
    }

    /// Atempts to find a place to pick up or drop off an item.
    ///
    /// If the `purpose` is [`Purpose::Primary`], items will not be picked up from or dropped off at a [`StorageInventory`].
    fn find_place_for_item(
        item_kind: ItemKind,
        delivery_mode: DeliveryMode,
        purpose: Purpose,
        unit_tile_pos: TilePos,
        facing: &Facing,
        goal: &Goal,
        input_inventory_query: &Query<&InputInventory, Without<MarkedForDemolition>>,
        output_inventory_query: &Query<&OutputInventory>,
        storage_inventory_query: &Query<&StorageInventory>,
        signals: &Signals,
        rng: &mut ThreadRng,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let neighboring_tiles = unit_tile_pos.reachable_neighbors(map_geometry);
        let mut candidates: Vec<(Entity, TilePos)> = Vec::new();

        for tile_pos in neighboring_tiles {
            for candidate in map_geometry.get_candidates(tile_pos, delivery_mode) {
                match (delivery_mode, purpose) {
                    (DeliveryMode::PickUp, Purpose::Intrinsic) => {
                        if let Ok(output_inventory) = output_inventory_query.get(candidate) {
                            if output_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }
                    }
                    (DeliveryMode::PickUp, Purpose::Instrumental) => {
                        if let Ok(output_inventory) = output_inventory_query.get(candidate) {
                            if output_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }

                        if let Ok(storage_inventory) = storage_inventory_query.get(candidate) {
                            if storage_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }
                    }
                    (DeliveryMode::DropOff, Purpose::Intrinsic) => {
                        if let Ok(input_inventory) = input_inventory_query.get(candidate) {
                            if input_inventory.currently_accepts(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }
                    }
                    (DeliveryMode::DropOff, Purpose::Instrumental) => {
                        if let Ok(input_inventory) = input_inventory_query.get(candidate) {
                            if input_inventory.currently_accepts(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }

                        if let Ok(storage_inventory) = storage_inventory_query.get(candidate) {
                            if storage_inventory.currently_accepts(item_kind, item_manifest) {
                                candidates.push((candidate, tile_pos));
                            }
                        }
                    }
                }
            }
        }

        if let Some((entity, tile_pos)) = candidates.choose(rng) {
            match delivery_mode {
                DeliveryMode::PickUp => {
                    CurrentAction::pickup(item_kind, *entity, facing, unit_tile_pos, *tile_pos)
                }
                DeliveryMode::DropOff => {
                    CurrentAction::dropoff(item_kind, *entity, facing, unit_tile_pos, *tile_pos)
                }
            }
        } else if let Some(upstream) =
            signals.upstream(unit_tile_pos, goal, item_manifest, map_geometry)
        {
            CurrentAction::move_or_spin(
                unit_tile_pos,
                upstream,
                facing,
                terrain_query,
                terrain_manifest,
                map_geometry,
            )
        } else {
            CurrentAction::idle()
        }
    }

    /// Attempt to find something that matches `workplace_id` to perform work
    fn find_workplace(
        workplace_id: WorkplaceId,
        unit_tile_pos: TilePos,
        facing: &Facing,
        workplace_query: &WorkplaceQuery,
        signals: &Signals,
        rng: &mut ThreadRng,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        item_manifest: &ItemManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let ahead = unit_tile_pos.neighbor(facing.direction);
        if let Some(workplace) =
            workplace_query.needs_work(unit_tile_pos, ahead, workplace_id, map_geometry)
        {
            CurrentAction::work(workplace)
        // Let units work even if they're standing on the structure
        // This is particularly relevant in the case of ghosts, where it's easy enough to end up on top of the structure trying to work on it
        } else if let Some(workplace) =
            workplace_query.needs_work(unit_tile_pos, unit_tile_pos, workplace_id, map_geometry)
        {
            CurrentAction::work(workplace)
        } else {
            let neighboring_tiles = unit_tile_pos.reachable_neighbors(map_geometry);
            let mut workplaces: Vec<(Entity, TilePos)> = Vec::new();

            for neighbor in neighboring_tiles {
                if let Some(workplace) =
                    workplace_query.needs_work(unit_tile_pos, neighbor, workplace_id, map_geometry)
                {
                    workplaces.push((workplace, neighbor));
                }
            }

            if let Some(chosen_workplace) = workplaces.choose(rng) {
                CurrentAction::move_or_spin(
                    unit_tile_pos,
                    chosen_workplace.1,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else if let Some(upstream) = signals.upstream(
                unit_tile_pos,
                &Goal::Work(workplace_id),
                item_manifest,
                map_geometry,
            ) {
                CurrentAction::move_or_spin(
                    unit_tile_pos,
                    upstream,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else {
                CurrentAction::idle()
            }
        }
    }

    /// Attempt to find a structure of type `structure_id` to perform work
    fn find_demolition_site(
        structure_id: Id<Structure>,
        unit_tile_pos: TilePos,
        facing: &Facing,
        demolition_query: &DemolitionQuery,
        signals: &Signals,
        rng: &mut ThreadRng,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let ahead = unit_tile_pos.neighbor(facing.direction);
        if let Some(workplace) =
            demolition_query.needs_demolition(unit_tile_pos, ahead, structure_id, map_geometry)
        {
            CurrentAction::demolish(workplace)
        } else if let Some(workplace) = demolition_query.needs_demolition(
            unit_tile_pos,
            unit_tile_pos,
            structure_id,
            map_geometry,
        ) {
            CurrentAction::demolish(workplace)
        } else {
            let neighboring_tiles = unit_tile_pos.reachable_neighbors(map_geometry);
            let mut demo_sites: Vec<(Entity, TilePos)> = Vec::new();

            for neighbor in neighboring_tiles {
                if let Some(demo_site) = demolition_query.needs_demolition(
                    unit_tile_pos,
                    neighbor,
                    structure_id,
                    map_geometry,
                ) {
                    demo_sites.push((demo_site, neighbor));
                }
            }

            if let Some(chosen_demo_site) = demo_sites.choose(rng) {
                CurrentAction::move_or_spin(
                    unit_tile_pos,
                    chosen_demo_site.1,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else if let Some(upstream) = signals.upstream(
                unit_tile_pos,
                &Goal::Demolish(structure_id),
                item_manifest,
                map_geometry,
            ) {
                CurrentAction::move_or_spin(
                    unit_tile_pos,
                    upstream,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else {
                CurrentAction::idle()
            }
        }
    }

    /// Spins 60 degrees left or right.
    pub(super) fn spin(rotation_direction: RotationDirection) -> Self {
        CurrentAction {
            action: UnitAction::Spin { rotation_direction },
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            just_started: true,
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
        current_tile: TilePos,
        facing: &Facing,
        map_geometry: &MapGeometry,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
    ) -> Self {
        /// The time in seconds that it takes a standard unit to walk to an adjacent tile.
        const BASE_WALKING_DURATION: f32 = 0.5;

        let target_tile = current_tile.neighbor(facing.direction);
        let entity_standing_on = map_geometry.get_terrain(current_tile).unwrap();
        let terrain_standing_on = terrain_query.get(entity_standing_on).unwrap();
        let walking_speed = terrain_manifest.get(*terrain_standing_on).walking_speed;
        let walking_duration = BASE_WALKING_DURATION / walking_speed;

        if map_geometry.is_passable(current_tile, target_tile) {
            CurrentAction {
                action: UnitAction::MoveForward,
                timer: Timer::from_seconds(walking_duration, TimerMode::Once),
                just_started: true,
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
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> Self {
        let required_direction = unit_tile_pos.direction_to(target_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction::move_forward(
                unit_tile_pos,
                facing,
                map_geometry,
                terrain_query,
                terrain_manifest,
            )
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Wait, as there is nothing to be done.
    pub(super) fn idle() -> Self {
        CurrentAction {
            action: UnitAction::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            just_started: true,
        }
    }

    /// Picks up the `item_id` at the `output_entity`.
    pub(super) fn pickup(
        item_kind: ItemKind,
        output_entity: Entity,
        facing: &Facing,
        unit_tile_pos: TilePos,
        output_tile_pos: TilePos,
    ) -> Self {
        let required_direction = unit_tile_pos.direction_to(output_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction {
                action: UnitAction::PickUp {
                    item_kind,
                    output_entity,
                },
                timer: Timer::from_seconds(0.5, TimerMode::Once),
                just_started: true,
            }
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Drops off the `item_id` at the `input_entity`.
    pub(super) fn dropoff(
        item_kind: ItemKind,
        input_entity: Entity,
        facing: &Facing,
        unit_tile_pos: TilePos,
        input_tile_pos: TilePos,
    ) -> Self {
        let required_direction = unit_tile_pos.direction_to(input_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction {
                action: UnitAction::DropOff {
                    item_kind,
                    input_entity,
                },
                timer: Timer::from_seconds(0.2, TimerMode::Once),
                just_started: true,
            }
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Eats the currently held item.
    pub(super) fn eat() -> Self {
        CurrentAction {
            action: UnitAction::Eat,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            just_started: true,
        }
    }

    /// Work at the specified structure
    pub(super) fn work(structure_entity: Entity) -> Self {
        CurrentAction {
            action: UnitAction::Work { structure_entity },
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            just_started: true,
        }
    }

    /// Demolish the specified structure
    pub(super) fn demolish(structure_entity: Entity) -> Self {
        CurrentAction {
            action: UnitAction::Demolish { structure_entity },
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            just_started: true,
        }
    }

    /// Drops the currently held item on the ground.
    ///
    /// If we cannot, wander around instead.
    pub(super) fn abandon(
        previous_action: UnitAction,
        unit_tile_pos: TilePos,
        unit_inventory: &UnitInventory,
        map_geometry: &MapGeometry,
        item_manifest: &ItemManifest,
        terrain_storage_query: &Query<&StorageInventory, With<Id<Terrain>>>,
        terrain_manifest: &TerrainManifest,
        terrain_query: &Query<&Id<Terrain>>,
        facing: &Facing,
        rng: &mut ThreadRng,
    ) -> Self {
        let terrain_entity = map_geometry.get_terrain(unit_tile_pos).unwrap();
        let terrain_storage_inventory = terrain_storage_query.get(terrain_entity).unwrap();

        if let Some(item_id) = unit_inventory.held_item {
            let item_kind = ItemKind::Single(item_id);
            if terrain_storage_inventory.currently_accepts(item_kind, item_manifest) {
                return CurrentAction {
                    action: UnitAction::Abandon,
                    timer: Timer::from_seconds(0.1, TimerMode::Once),
                    just_started: true,
                };
            }
        }

        CurrentAction::wander(
            previous_action,
            unit_tile_pos,
            facing,
            map_geometry,
            terrain_query,
            terrain_manifest,
            rng,
        )
    }

    /// Wander around randomly.
    ///
    /// This will alternate between moving forward and spinning.
    pub(super) fn wander(
        previous_action: UnitAction,
        unit_tile_pos: TilePos,
        facing: &Facing,
        map_geometry: &MapGeometry,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        rng: &mut ThreadRng,
    ) -> Self {
        match previous_action {
            UnitAction::Spin { .. } => CurrentAction::move_forward(
                unit_tile_pos,
                facing,
                map_geometry,
                &terrain_query,
                &terrain_manifest,
            ),
            _ => CurrentAction::random_spin(rng),
        }
    }
}

/// A query about the [`CraftingState`] of a structure that might need work done.
#[derive(SystemParam)]
pub(crate) struct WorkplaceQuery<'w, 's> {
    /// The contained query type.
    query: Query<
        'w,
        's,
        (
            &'static CraftingState,
            AnyOf<(&'static Id<Structure>, &'static TerraformingAction)>,
            &'static WorkersPresent,
        ),
    >,
}

impl<'w, 's> WorkplaceQuery<'w, 's> {
    /// Is there a structure of type `structure_id` at `structure_pos` that needs work done by a unit?
    ///
    /// If so, returns `Some(matching_structure_entity_that_needs_work)`.
    pub(crate) fn needs_work(
        &self,
        current: TilePos,
        target: TilePos,
        workplace_id: WorkplaceId,
        map_geometry: &MapGeometry,
    ) -> Option<Entity> {
        // This is only a viable target if the unit can reach it!
        let height_difference = map_geometry.height_difference(current, target).ok()?;
        if height_difference > Height::MAX_STEP {
            return None;
        }

        let entity = *map_geometry.get_workplaces(target).first()?;

        let (found_crafting_state, ids, workers_present) = self.query.get(entity).ok()?;

        if workplace_id != WorkplaceId::new(ids) {
            return None;
        }

        if let CraftingState::InProgress { .. } = found_crafting_state {
            if workers_present.needs_more() {
                Some(entity)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Are units attempting to find or deliver items?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DeliveryMode {
    /// Units are attempting to find items.
    PickUp,
    /// Units are attempting to deliver items.
    DropOff,
}

/// Is this action essential to our ultimate goal, or can we be flexible?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Purpose {
    /// This action is essential to the goal.
    ///
    /// This will only take / place items from active sources of signals.
    Intrinsic,
    /// This action is not essential to the goal.
    ///
    /// This will take / place items from storage.
    Instrumental,
}
