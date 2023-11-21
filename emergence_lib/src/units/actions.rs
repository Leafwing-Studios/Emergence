//! What are units currently doing?

use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
    utils::Duration,
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
        inventories::{
            AddToInputError, CraftingState, InputInventory, OutputInventory, StorageInventory,
        },
        item_tags::ItemKind,
        workers::WorkersPresent,
    },
    geometry::{Facing, Height, MapGeometry, RotationDirection, VoxelPos},
    items::{errors::AddOneItemError, item_manifest::ItemManifest, ItemCount},
    litter::{Litter, LitterCommandsExt},
    organisms::{energy::EnergyPool, lifecycle::Lifecycle},
    signals::{SignalType, Signals},
    structures::{commands::StructureCommandsExt, structure_manifest::Structure},
    terrain::terrain_manifest::{Terrain, TerrainManifest},
    water::WaterDepth,
};

use super::{
    goals::Goal,
    impatience::ImpatiencePool,
    item_interaction::UnitInventory,
    unit_manifest::{Unit, UnitManifest},
};

/// Ticks the timer for each [`CurrentAction`].
pub(super) fn advance_action_timer(mut units_query: Query<&mut CurrentAction>, time: Res<Time>) {
    let delta = time.delta();

    for mut current_action in units_query.iter_mut() {
        current_action.timer.tick(delta);
    }
}

/// Choose the unit's action for this turn
pub(super) fn choose_actions(
    mut units_query: Query<
        (
            &VoxelPos,
            &Facing,
            &Goal,
            &mut CurrentAction,
            &UnitInventory,
        ),
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
    litter_query: Query<&Litter>,
    water_depth_query: Query<&WaterDepth>,
    terrain_manifest: Res<TerrainManifest>,
    item_manifest: Res<ItemManifest>,
) {
    let rng = &mut thread_rng();

    for (&unit_pos, facing, goal, mut current_action, unit_inventory) in units_query.iter_mut() {
        if current_action.finished() {
            let previous_action = current_action.action.clone();

            *current_action = match goal {
                // Drop whatever you're holding before wandering further
                Goal::Wander { .. } => match unit_inventory.held_item {
                    Some(_) => CurrentAction::abandon(
                        previous_action,
                        unit_pos,
                        unit_inventory,
                        &map_geometry,
                        &terrain_manifest,
                        &terrain_query,
                        rng,
                    ),
                    None => CurrentAction::wander(
                        previous_action,
                        unit_pos,
                        &map_geometry,
                        &terrain_query,
                        &terrain_manifest,
                        rng,
                    ),
                },
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
                            unit_pos,
                            unit_inventory,
                            &map_geometry,
                            &terrain_manifest,
                            &terrain_query,
                            rng,
                        )
                    } else {
                        CurrentAction::find(
                            unit_inventory,
                            *item_kind,
                            goal.delivery_mode().unwrap(),
                            goal.purpose(),
                            unit_pos,
                            facing,
                            goal,
                            &input_inventory_query,
                            &output_inventory_query,
                            &storage_inventory_query,
                            &litter_query,
                            &signals,
                            rng,
                            &item_manifest,
                            &terrain_query,
                            &terrain_manifest,
                            &map_geometry,
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
                                unit_pos,
                                unit_inventory,
                                &map_geometry,
                                &terrain_manifest,
                                &terrain_query,
                                rng,
                            )
                        }
                    } else {
                        CurrentAction::find(
                            unit_inventory,
                            *item_kind,
                            DeliveryMode::PickUp,
                            Purpose::Instrumental,
                            unit_pos,
                            facing,
                            goal,
                            &input_inventory_query,
                            &output_inventory_query,
                            &storage_inventory_query,
                            &litter_query,
                            &signals,
                            rng,
                            &item_manifest,
                            &terrain_query,
                            &terrain_manifest,
                            &map_geometry,
                        )
                    }
                }
                Goal::Work(structure_id) => CurrentAction::find_workplace(
                    *structure_id,
                    unit_pos,
                    facing,
                    &workplace_query,
                    &signals,
                    rng,
                    &terrain_query,
                    &terrain_manifest,
                    &item_manifest,
                    &map_geometry,
                ),
                Goal::Demolish(structure_id) => CurrentAction::find_demolition_site(
                    *structure_id,
                    unit_pos,
                    facing,
                    &demolition_query,
                    &signals,
                    rng,
                    &item_manifest,
                    &terrain_query,
                    &terrain_manifest,
                    &map_geometry,
                ),
                Goal::Avoid(unit_id) => CurrentAction::avoid(
                    *unit_id,
                    unit_pos,
                    facing,
                    &signals,
                    &item_manifest,
                    &terrain_query,
                    &terrain_manifest,
                    &map_geometry,
                ),
                Goal::Breathe => CurrentAction::find_oxygen(
                    unit_pos,
                    facing,
                    &water_depth_query,
                    &terrain_query,
                    &terrain_manifest,
                    &map_geometry,
                    rng,
                ),
            }
        }
    }
}

/// Exhaustively handles the setup for each planned action
pub(super) fn start_actions(
    mut unit_query: Query<(Entity, &mut CurrentAction)>,
    mut workplace_query: Query<&mut WorkersPresent>,
) {
    for (worker_entity, mut action) in unit_query.iter_mut() {
        if action.just_started {
            if let Some(workplace_entity) = action.action().workplace() {
                if let Ok(mut workers_present) = workplace_query.get_mut(workplace_entity) {
                    // This has a side effect of adding the worker to the workplace
                    let result = workers_present.add_worker(worker_entity);
                    if result.is_err() {
                        *action = CurrentAction::idle();
                    }
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
            &mut Litter,
        )>,
    >,
    mut workplace_query: Query<(&CraftingState, &mut WorkersPresent)>,
    // This must be compatible with unit_query
    structure_query: Query<&VoxelPos, (With<Id<Structure>>, Without<Goal>)>,
    item_manifest: Res<ItemManifest>,
    unit_manifest: Res<UnitManifest>,
    signals: Res<Signals>,
    map_geometry: Res<MapGeometry>,
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
                    workers_present.remove_worker(unit.entity);
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
                    if let Ok((
                        _,
                        mut maybe_output_inventory,
                        mut maybe_storage_inventory,
                        mut maybe_litter,
                    )) = inventory_query.get_mut(*output_entity)
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
                                } else if let Some(ref litter) = maybe_litter {
                                    litter.matching_item_id(*item_kind, item_manifest)
                                } else {
                                    // The entity must have either an output, storage or litter inventory
                                    unreachable!()
                                };

                                if let Some(item_id) = maybe_item_id {
                                    let item_count = ItemCount::new(item_id, 1);

                                    let transfer_result = match (
                                        &mut maybe_output_inventory,
                                        &mut maybe_storage_inventory,
                                        &mut maybe_litter,
                                    ) {
                                        (Some(ref mut output_inventory), _, _) => {
                                            output_inventory.remove_item_all_or_nothing(&item_count)
                                        }
                                        (_, Some(ref mut storage_inventory), _) => {
                                            storage_inventory
                                                .remove_item_all_or_nothing(&item_count)
                                        }
                                        (_, _, Some(ref mut litter)) => {
                                            litter.remove_item_all_or_nothing(&item_count)
                                        }
                                        // The entity must have either an output, storage or litter inventory
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
                                                *unit.voxel_pos,
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
                    if let Ok((maybe_input_inventory, _, maybe_storage_inventory, _)) =
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
                    RotationDirection::Left => unit.facing.rotate_counterclockwise(),
                    RotationDirection::Right => unit.facing.rotate_clockwise(),
                },
                UnitAction::MoveForward => {
                    if let Some(target_voxel) = map_geometry
                        .walkable_neighbor_in_direction(*unit.voxel_pos, unit.facing.direction)
                    {
                        *unit.voxel_pos = target_voxel;
                        unit.transform.translation = target_voxel.inside_voxel();
                    } else {
                        warn!(
                            "Unit {:?} tried to move forward but no walkable voxel in direction {:?}",
                            unit.entity, unit.facing.direction
                        );
                    }
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
                        // FIXME: this doesn't work for structures that don't cover the origin of their footprint
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
                    if let Some(held_item) = unit.unit_inventory.held_item {
                        commands.spawn_litter(*unit.voxel_pos, held_item);
                        unit.unit_inventory.held_item = None;
                    } else {
                        unit.impatience.increment();
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
    /// The [`Entity`] of the acting unit
    entity: Entity,
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
    voxel_pos: &'static mut VoxelPos,
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

    /// The amount of time it takes to complete this action.
    // PERF: this needs nightly to be const
    fn duration(&self) -> Duration {
        let seconds = match self {
            UnitAction::PickUp { .. } => 0.2,
            UnitAction::DropOff { .. } => 0.2,
            UnitAction::Abandon => 0.2,
            UnitAction::Work { .. } => 0.1,
            UnitAction::Demolish { .. } => 0.1,
            UnitAction::Eat => 0.3,
            UnitAction::Idle => 0.1,
            UnitAction::Spin { .. } => 0.1,
            UnitAction::MoveForward => 0.3,
        };

        Duration::from_secs_f32(seconds)
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
    /// Creates a new action with the default duration.
    fn new(action: UnitAction) -> Self {
        let duration = action.duration();
        Self {
            action,
            timer: Timer::new(duration, TimerMode::Once),
            just_started: true,
        }
    }

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
    /// If the `purpose` is [`Purpose::Intrinsic`], items will not be picked up from or dropped off at a [`StorageInventory`].
    /// The only exception is if the storage inventory is full, in which case the unit will pick up items from there.
    ///
    /// Items will never be dropped off at litter, and will only be picked up from litter if no other local options are available.
    fn find(
        unit_inventory: &UnitInventory,
        item_kind: ItemKind,
        delivery_mode: DeliveryMode,
        purpose: Purpose,
        unit_pos: VoxelPos,
        facing: &Facing,
        goal: &Goal,
        input_inventory_query: &Query<&InputInventory, Without<MarkedForDemolition>>,
        output_inventory_query: &Query<&OutputInventory>,
        storage_inventory_query: &Query<&StorageInventory>,
        litter_query: &Query<&Litter>,
        signals: &Signals,
        rng: &mut ThreadRng,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let mut candidates: Vec<(Entity, VoxelPos)> = Vec::new();
        let held_item = unit_inventory.held_item;

        // If we're not holding anyhing, we can't drop it off
        if held_item.is_none() && delivery_mode == DeliveryMode::DropOff {
            return CurrentAction::idle();
        }

        for voxel_pos in unit_pos.reachable_neighbors() {
            if let Some(candidate) = map_geometry.get_candidate(voxel_pos, delivery_mode) {
                match (delivery_mode, purpose) {
                    (DeliveryMode::PickUp, Purpose::Intrinsic) => {
                        if let Ok(output_inventory) = output_inventory_query.get(candidate) {
                            if output_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, voxel_pos));
                            }
                        }

                        if let Ok(storage_inventory) = storage_inventory_query.get(candidate) {
                            if storage_inventory.is_full()
                                && storage_inventory.contains_kind(item_kind, item_manifest)
                            {
                                candidates.push((candidate, voxel_pos));
                            }
                        }
                    }
                    (DeliveryMode::PickUp, Purpose::Instrumental) => {
                        if let Ok(output_inventory) = output_inventory_query.get(candidate) {
                            if output_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, voxel_pos));
                            }
                        }

                        if let Ok(storage_inventory) = storage_inventory_query.get(candidate) {
                            if storage_inventory.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, voxel_pos));
                            }
                        }

                        if let Ok(litter) = litter_query.get(candidate) {
                            if litter.contains_kind(item_kind, item_manifest) {
                                candidates.push((candidate, voxel_pos));
                            }
                        }
                    }
                    (DeliveryMode::DropOff, Purpose::Intrinsic) => {
                        if let Ok(input_inventory) = input_inventory_query.get(candidate) {
                            if input_inventory.currently_accepts(held_item.unwrap(), item_manifest)
                            {
                                candidates.push((candidate, voxel_pos));
                            }
                        }
                    }
                    (DeliveryMode::DropOff, Purpose::Instrumental) => {
                        if let Ok(input_inventory) = input_inventory_query.get(candidate) {
                            if input_inventory.currently_accepts(held_item.unwrap(), item_manifest)
                            {
                                candidates.push((candidate, voxel_pos));
                            }
                        }

                        if let Ok(storage_inventory) = storage_inventory_query.get(candidate) {
                            if storage_inventory
                                .currently_accepts(held_item.unwrap(), item_manifest)
                            {
                                candidates.push((candidate, voxel_pos));
                            }
                        }
                    }
                }
            }
        }

        if let Some((entity, voxel_pos)) = candidates.choose(rng) {
            match delivery_mode {
                DeliveryMode::PickUp => {
                    CurrentAction::pickup(item_kind, *entity, facing, unit_pos, *voxel_pos)
                }
                DeliveryMode::DropOff => {
                    CurrentAction::dropoff(item_kind, *entity, facing, unit_pos, *voxel_pos)
                }
            }
        } else if let Some(upstream) = signals.upstream(unit_pos, goal, item_manifest, map_geometry)
        {
            CurrentAction::move_or_spin(
                unit_pos,
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
        unit_pos: VoxelPos,
        facing: &Facing,
        workplace_query: &WorkplaceQuery,
        signals: &Signals,
        rng: &mut ThreadRng,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        item_manifest: &ItemManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let ahead = unit_pos.neighbor(facing.direction);
        if let Some(workplace) =
            workplace_query.needs_work(unit_pos, ahead, workplace_id, map_geometry)
        {
            CurrentAction::work(workplace)
        // Let units work even if they're standing on the structure
        // This is particularly relevant in the case of ghosts, where it's easy enough to end up on top of the structure trying to work on it
        } else if let Some(workplace) =
            workplace_query.needs_work(unit_pos, unit_pos, workplace_id, map_geometry)
        {
            CurrentAction::work(workplace)
        } else {
            let mut workplaces: Vec<(Entity, VoxelPos)> = Vec::new();

            for neighbor in unit_pos.reachable_neighbors() {
                if let Some(workplace) =
                    workplace_query.needs_work(unit_pos, neighbor, workplace_id, map_geometry)
                {
                    workplaces.push((workplace, neighbor));
                }
            }

            if let Some(chosen_workplace) = workplaces.choose(rng) {
                CurrentAction::move_or_spin(
                    unit_pos,
                    chosen_workplace.1,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else if let Some(upstream) = signals.upstream(
                unit_pos,
                &Goal::Work(workplace_id),
                item_manifest,
                map_geometry,
            ) {
                CurrentAction::move_or_spin(
                    unit_pos,
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
        unit_pos: VoxelPos,
        facing: &Facing,
        demolition_query: &DemolitionQuery,
        signals: &Signals,
        rng: &mut ThreadRng,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> CurrentAction {
        let ahead = unit_pos.neighbor(facing.direction);
        if let Some(workplace) =
            demolition_query.needs_demolition(unit_pos, ahead, structure_id, map_geometry)
        {
            CurrentAction::demolish(workplace)
        } else if let Some(workplace) =
            demolition_query.needs_demolition(unit_pos, unit_pos, structure_id, map_geometry)
        {
            CurrentAction::demolish(workplace)
        } else {
            let mut demo_sites: Vec<(Entity, VoxelPos)> = Vec::new();

            for neighbor in unit_pos.reachable_neighbors() {
                if let Some(demo_site) = demolition_query.needs_demolition(
                    unit_pos,
                    neighbor,
                    structure_id,
                    map_geometry,
                ) {
                    demo_sites.push((demo_site, neighbor));
                }
            }

            if let Some(chosen_demo_site) = demo_sites.choose(rng) {
                CurrentAction::move_or_spin(
                    unit_pos,
                    chosen_demo_site.1,
                    facing,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                )
            } else if let Some(upstream) = signals.upstream(
                unit_pos,
                &Goal::Demolish(structure_id),
                item_manifest,
                map_geometry,
            ) {
                CurrentAction::move_or_spin(
                    unit_pos,
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
        CurrentAction::new(UnitAction::Spin { rotation_direction })
    }

    /// Rotate to face the `required_direction`.
    fn spin_towards(facing: &Facing, required_direction: hexx::Direction) -> Self {
        let mut working_direction_left = facing.direction;
        let mut working_direction_right = facing.direction;

        // Let's race!
        // Left gets an arbitrary unfair advantage though.
        // PERF: this could use a lookup table instead, and would probably be faster
        loop {
            working_direction_left = working_direction_left.counter_clockwise();
            if working_direction_left == required_direction {
                return CurrentAction::spin(RotationDirection::Left);
            }

            working_direction_right = working_direction_right.clockwise();
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

    /// Move away from the signals matching the provided `goal` if able.
    pub(super) fn move_away_from(
        goal: &Goal,
        current_tile: VoxelPos,
        facing: &Facing,
        signals: &Signals,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> Self {
        if let Some(target_tile) =
            signals.downstream(current_tile, goal, item_manifest, map_geometry)
        {
            CurrentAction::move_or_spin(
                current_tile,
                target_tile,
                facing,
                terrain_query,
                terrain_manifest,
                map_geometry,
            )
        } else {
            CurrentAction::idle()
        }
    }

    /// Move toward the tile this unit is facing if able
    pub(super) fn move_forward(
        current_voxel: VoxelPos,
        map_geometry: &MapGeometry,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
    ) -> Self {
        /// The multiplier applied to the walking speed when walking on a path.
        // TODO: vary this based on the path type
        const PATH_MULTIPLIER: f32 = 1.5;

        let entity_standing_on = map_geometry.get_terrain(current_voxel.hex).unwrap();
        let walking_speed = if map_geometry.get_structure(current_voxel).is_some() {
            PATH_MULTIPLIER
        } else {
            let terrain_standing_on = terrain_query.get(entity_standing_on).unwrap();
            terrain_manifest.get(*terrain_standing_on).walking_speed
        };

        let walking_duration = UnitAction::MoveForward.duration().as_secs_f32() / walking_speed;

        CurrentAction {
            action: UnitAction::MoveForward,
            timer: Timer::from_seconds(walking_duration, TimerMode::Once),
            just_started: true,
        }
    }

    /// Attempt to move toward the `target_tile_pos`.
    pub(super) fn move_or_spin(
        unit_pos: VoxelPos,
        target_tile_pos: VoxelPos,
        facing: &Facing,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> Self {
        let required_direction = unit_pos.hex.main_direction_to(target_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction::move_forward(unit_pos, map_geometry, terrain_query, terrain_manifest)
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Wait, as there is nothing to be done.
    pub(super) fn idle() -> Self {
        CurrentAction::new(UnitAction::Idle)
    }

    /// Picks up the `item_id` at the `output_entity`.
    pub(super) fn pickup(
        item_kind: ItemKind,
        output_entity: Entity,
        facing: &Facing,
        unit_pos: VoxelPos,
        output_tile_pos: VoxelPos,
    ) -> Self {
        let required_direction = unit_pos.hex.main_direction_to(output_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction::new(UnitAction::PickUp {
                item_kind,
                output_entity,
            })
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Drops off the `item_id` at the `input_entity`.
    pub(super) fn dropoff(
        item_kind: ItemKind,
        input_entity: Entity,
        facing: &Facing,
        unit_pos: VoxelPos,
        input_tile_pos: VoxelPos,
    ) -> Self {
        let required_direction = unit_pos.hex.main_direction_to(input_tile_pos.hex);

        if required_direction == facing.direction {
            CurrentAction::new(UnitAction::DropOff {
                item_kind,
                input_entity,
            })
        } else {
            CurrentAction::spin_towards(facing, required_direction)
        }
    }

    /// Eats the currently held item.
    pub(super) fn eat() -> Self {
        CurrentAction::new(UnitAction::Eat)
    }

    /// Work at the specified structure
    pub(super) fn work(structure_entity: Entity) -> Self {
        CurrentAction::new(UnitAction::Work { structure_entity })
    }

    /// Demolish the specified structure
    pub(super) fn demolish(structure_entity: Entity) -> Self {
        CurrentAction::new(UnitAction::Demolish { structure_entity })
    }

    /// Drops the currently held item on the ground.
    ///
    /// If we cannot, wander around instead.
    pub(super) fn abandon(
        previous_action: UnitAction,
        unit_pos: VoxelPos,
        unit_inventory: &UnitInventory,
        map_geometry: &MapGeometry,
        terrain_manifest: &TerrainManifest,
        terrain_query: &Query<&Id<Terrain>>,
        rng: &mut ThreadRng,
    ) -> Self {
        if unit_inventory.held_item.is_some() {
            CurrentAction::new(UnitAction::Abandon)
        } else {
            CurrentAction::wander(
                previous_action,
                unit_pos,
                map_geometry,
                terrain_query,
                terrain_manifest,
                rng,
            )
        }
    }

    /// Wander around randomly.
    ///
    /// This will alternate between moving forward and spinning.
    pub(super) fn wander(
        previous_action: UnitAction,
        unit_pos: VoxelPos,
        map_geometry: &MapGeometry,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        rng: &mut ThreadRng,
    ) -> Self {
        match previous_action {
            UnitAction::Spin { .. } => {
                CurrentAction::move_forward(unit_pos, map_geometry, terrain_query, terrain_manifest)
            }
            _ => CurrentAction::random_spin(rng),
        }
    }

    /// Flee a [`SignalType::Unit`] signal matching `unit_id`.
    fn avoid(
        unit_id: Id<Unit>,
        current_tile: VoxelPos,
        facing: &Facing,
        signals: &Signals,
        item_manifest: &ItemManifest,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> Self {
        /// The relative signal strength threshold at which we will stop avoiding the source of our discomfort.
        ///
        /// Increasing this value will make units avoid for longer.
        /// To increase the frequency at which this goal is chosen at all, change the signal strength instead.
        ///
        /// This should be a value between 0 and 1.
        const SIGNAL_STRENGTH_THRESHOLD: f32 = 0.5;

        let avoided_signal_strength = signals.get(SignalType::Unit(unit_id), current_tile);

        // If our signal is more than some fraction as strong as the strongest other signal, then keep moving.
        let strongest_signal = signals.strongest_goal_signal_at_position(current_tile);
        if let Some((_, strongest_signal_strength)) = strongest_signal {
            if avoided_signal_strength > strongest_signal_strength * SIGNAL_STRENGTH_THRESHOLD {
                return CurrentAction::move_away_from(
                    &Goal::Avoid(unit_id),
                    current_tile,
                    facing,
                    signals,
                    item_manifest,
                    terrain_query,
                    terrain_manifest,
                    map_geometry,
                );
            }
        }

        // Otherwise, idle.
        CurrentAction::idle()
    }

    /// Attempts to move to shallower water.
    fn find_oxygen(
        current_tile: VoxelPos,
        facing: &Facing,
        water_depth_query: &Query<&WaterDepth>,
        terrain_query: &Query<&Id<Terrain>>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
        rng: &mut ThreadRng,
    ) -> Self {
        let terrain_entity = map_geometry.get_terrain(current_tile.hex).unwrap();
        let current_depth = water_depth_query
            .get(terrain_entity)
            .unwrap()
            .surface_water_depth();
        let mut candidates = Vec::new();

        // Find all adjacent tiles that are shallower than the current tile.
        for adjacent_tile in map_geometry.walkable_neighbors(current_tile) {
            let adjacent_terrain_entity = map_geometry.get_terrain(current_tile.hex).unwrap();
            let adjacent_depth = water_depth_query
                .get(adjacent_terrain_entity)
                .unwrap()
                .surface_water_depth();

            if adjacent_depth < current_depth {
                candidates.push(adjacent_tile);
            }
        }

        // Pick a random candidate.
        if let Some(target_tile) = candidates.choose(rng) {
            CurrentAction::move_or_spin(
                current_tile,
                *target_tile,
                facing,
                terrain_query,
                terrain_manifest,
                map_geometry,
            )
        } else {
            CurrentAction::random_spin(rng)
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
        current: VoxelPos,
        target: VoxelPos,
        workplace_id: WorkplaceId,
        map_geometry: &MapGeometry,
    ) -> Option<Entity> {
        // This is only a viable target if the unit can reach it!
        if current.abs_height_diff(target) > Height::MAX_STEP {
            return None;
        }

        let entity = map_geometry.get_workplace(target)?;

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
