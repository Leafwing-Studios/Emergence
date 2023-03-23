//! Everything needed to make structures able to craft things.

use std::{fmt::Display, time::Duration};

use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};
use leafwing_abilities::prelude::Pool;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng};

use crate::{
    asset_management::manifest::{
        Id, Item, ItemManifest, Manifest, Recipe, RecipeManifest, Structure, StructureManifest,
    },
    items::{inventory::Inventory, recipe::RecipeData},
    organisms::{energy::EnergyPool, lifecycle::Lifecycle, Organism},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::{
        geometry::{MapGeometry, TilePos},
        light::TotalLight,
        SimulationSet,
    },
};

/// The current state in the crafting progress.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) enum CraftingState {
    /// There are resources missing for the recipe.
    #[default]
    NeedsInput,
    /// The resource cost has been paid and the recipe is being crafted.
    InProgress {
        /// How far through the recipe are we?
        progress: Duration,
        /// How long does this recipe take to complete in full?
        required: Duration,
    },
    /// Resources need to be claimed before more crafting can continue.
    FullAndBlocked,
    /// The recipe is complete.
    RecipeComplete,
    /// The output is full but production is continuing.
    Overproduction,
    /// No recipe is set
    NoRecipe,
}

impl Display for CraftingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            CraftingState::NeedsInput => "Waiting for input".to_string(),
            CraftingState::InProgress { progress, required } => {
                let progress_in_seconds = progress.as_secs_f32();
                let required_in_seconds = required.as_secs_f32();
                format!("In progress ({progress_in_seconds:.1} / {required_in_seconds:.1})")
            }
            CraftingState::RecipeComplete => "Recipe complete".to_string(),
            CraftingState::FullAndBlocked => "Blocked".to_string(),
            CraftingState::Overproduction => "Overproduction".to_string(),
            CraftingState::NoRecipe => "No recipe set".to_string(),
        };

        write!(f, "{string}")
    }
}

/// The input inventory for a structure.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub(crate) struct InputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

impl InputInventory {
    /// Randomizes the contents of this inventory so that each slot is somewhere between empty and full.
    pub(super) fn randomize(&mut self, rng: &mut ThreadRng) {
        for item_slot in self.iter_mut() {
            item_slot.randomize(rng);
        }
    }
}

/// The output inventory for a structure.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct OutputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

impl OutputInventory {
    /// Randomizes the contents of this inventory so that each slot is somewhere between empty and full.
    pub(super) fn randomize(&mut self, rng: &mut ThreadRng) {
        for item_slot in self.iter_mut() {
            item_slot.randomize(rng);
        }
    }
}

/// An inventory that simply stores items
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub(crate) struct StorageInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

impl StorageInventory {
    /// Creates a new [`StorageInventory`] with the provided number of slots.
    ///
    /// If `reserved_for` is `Some`, only one item variety will be able to be stored here.
    pub(crate) fn new(max_slot_count: usize, reserved_for: Option<Id<Item>>) -> Self {
        StorageInventory {
            inventory: Inventory::new(max_slot_count, reserved_for),
        }
    }
}

/// The recipe that is currently being crafted, if any.
#[derive(Component, Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct ActiveRecipe(Option<Id<Recipe>>);

impl ActiveRecipe {
    /// The un-set [`ActiveRecipe`].
    pub(crate) const NONE: ActiveRecipe = ActiveRecipe(None);

    /// Creates a new [`ActiveRecipe`], set to `recipe_id`
    pub(crate) fn new(recipe_id: Id<Recipe>) -> Self {
        ActiveRecipe(Some(recipe_id))
    }

    /// The ID of the currently active recipe, if one has been selected.
    pub(crate) fn recipe_id(&self) -> &Option<Id<Recipe>> {
        &self.0
    }

    /// The pretty formatting for this type
    pub(crate) fn display(&self, recipe_manifest: &RecipeManifest) -> String {
        match self.0 {
            Some(recipe_id) => recipe_manifest.name(recipe_id).to_string(),
            None => "None".to_string(),
        }
    }
}

/// All components needed to craft stuff.
#[derive(Debug, Bundle)]
pub(crate) struct CraftingBundle {
    /// The input inventory for the items needed for crafting.
    input_inventory: InputInventory,

    /// The output inventory for the crafted items.
    output_inventory: OutputInventory,

    /// The recipe that is currently being crafted.
    active_recipe: ActiveRecipe,

    /// The current state for the crafting process.
    craft_state: CraftingState,

    /// Emits signals, drawing units towards this structure to ensure crafting flows smoothly
    emitter: Emitter,

    /// The number of workers present / allowed at this structure
    workers_present: WorkersPresent,
}

/// The number of workers present / allowed at this structure.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkersPresent {
    /// The number of workers present
    present: u8,
    /// The maximum number of workers allowed
    allowed: u8,
}

impl WorkersPresent {
    /// Create a new [`WorkersPresent`] with the provided maximum number of workers allowed.
    pub(crate) fn new(allowed: u8) -> Self {
        Self {
            present: 0,
            allowed,
        }
    }

    /// Are more workers needed?
    pub(crate) fn needs_more(&self) -> bool {
        self.present < self.allowed
    }

    /// The number of workers present.
    pub(crate) fn current(&self) -> u8 {
        self.present
    }

    /// Adds a worker to this structure if there is room.
    pub(crate) fn add_worker(&mut self) -> Result<(), ()> {
        if self.needs_more() {
            self.present += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Removes a worker from this structure
    pub(crate) fn remove_worker(&mut self) {
        self.present = self.present.saturating_sub(1);
    }
}

impl Display for WorkersPresent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{present} / {allowed}",
            present = self.present,
            allowed = self.allowed
        )
    }
}

impl CraftingBundle {
    /// Create a new crafting bundle with empty inventories.
    pub(crate) fn new(
        structure_id: Id<Structure>,
        starting_recipe: ActiveRecipe,
        recipe_manifest: &RecipeManifest,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
    ) -> Self {
        let max_workers = structure_manifest.get(structure_id).max_workers;

        if let Some(recipe_id) = starting_recipe.0 {
            let recipe = recipe_manifest.get(recipe_id);

            Self {
                input_inventory: recipe.input_inventory(item_manifest),
                output_inventory: recipe.output_inventory(item_manifest),
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::NeedsInput,
                emitter: Emitter::default(),
                workers_present: WorkersPresent::new(max_workers),
            }
        } else {
            Self {
                input_inventory: InputInventory {
                    inventory: Inventory::new(0, None),
                },
                output_inventory: OutputInventory {
                    inventory: Inventory::new(1, None),
                },
                active_recipe: ActiveRecipe(None),
                craft_state: CraftingState::NeedsInput,
                emitter: Emitter::default(),
                workers_present: WorkersPresent::new(max_workers),
            }
        }
    }

    /// Generates a new crafting bundle that is at a random point in its cycle.
    pub(crate) fn randomized(
        structure_id: Id<Structure>,
        starting_recipe: ActiveRecipe,
        recipe_manifest: &RecipeManifest,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
        rng: &mut ThreadRng,
    ) -> Self {
        if let Some(recipe_id) = starting_recipe.0 {
            let recipe = recipe_manifest.get(recipe_id);

            let mut input_inventory = recipe.input_inventory(item_manifest);
            input_inventory.randomize(rng);
            let mut output_inventory = recipe.output_inventory(item_manifest);
            output_inventory.randomize(rng);

            let distribution = Uniform::new(Duration::ZERO, recipe.craft_time());
            let progress = distribution.sample(rng);
            let max_workers = structure_manifest.get(structure_id).max_workers;

            Self {
                input_inventory,
                output_inventory,
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::InProgress {
                    progress,
                    required: recipe.craft_time(),
                },
                emitter: Emitter::default(),
                workers_present: WorkersPresent::new(max_workers),
            }
        } else {
            CraftingBundle::new(
                structure_id,
                starting_recipe,
                recipe_manifest,
                item_manifest,
                structure_manifest,
            )
        }
    }
}

/// Data needed for [`progress_crafting`].
#[derive(WorldQuery)]
#[world_query(mutable)]
struct CraftingQuery {
    /// The recipe of the crafter
    active_recipe: &'static ActiveRecipe,
    /// The status of crafting
    state: &'static mut CraftingState,
    /// The inputs
    input: &'static mut InputInventory,
    /// The outputs
    output: &'static mut OutputInventory,
    /// The number of workers present
    workers_present: &'static WorkersPresent,
    /// Is this an organism?
    maybe_organism: Option<&'static Organism>,
}

/// Progress the state of recipes that are being crafted.
fn progress_crafting(
    time: Res<FixedTime>,
    recipe_manifest: Res<RecipeManifest>,
    item_manifest: Res<ItemManifest>,
    total_light: Res<TotalLight>,
    mut crafting_query: Query<CraftingQuery>,
) {
    for mut crafter in crafting_query.iter_mut() {
        *crafter.state = match *crafter.state {
            CraftingState::NoRecipe => match crafter.active_recipe.recipe_id() {
                Some(_) => CraftingState::NeedsInput,
                None => CraftingState::NoRecipe,
            },
            CraftingState::NeedsInput | CraftingState::Overproduction => {
                if let Some(recipe_id) = crafter.active_recipe.recipe_id() {
                    let recipe = recipe_manifest.get(*recipe_id);
                    match crafter.input.remove_items_all_or_nothing(recipe.inputs()) {
                        Ok(()) => CraftingState::InProgress {
                            progress: Duration::ZERO,
                            required: recipe.craft_time(),
                        },
                        Err(_) => CraftingState::NeedsInput,
                    }
                } else {
                    CraftingState::NoRecipe
                }
            }
            CraftingState::InProgress { progress, required } => {
                let mut updated_progress = progress;
                if let Some(recipe_id) = crafter.active_recipe.recipe_id() {
                    let recipe = recipe_manifest.get(*recipe_id);
                    if recipe.satisfied(crafter.workers_present.current(), &total_light) {
                        updated_progress += time.period;
                        if updated_progress >= required {
                            CraftingState::RecipeComplete
                        } else {
                            CraftingState::InProgress {
                                progress: updated_progress,
                                required,
                            }
                        }
                    } else {
                        CraftingState::InProgress { progress, required }
                    }
                } else {
                    CraftingState::NoRecipe
                }
            }
            CraftingState::RecipeComplete => {
                if let Some(recipe_id) = crafter.active_recipe.recipe_id() {
                    let recipe = recipe_manifest.get(*recipe_id);
                    match crafter.maybe_organism {
                        Some(_) => {
                            match crafter
                                .output
                                .try_add_items(recipe.outputs(), &item_manifest)
                            {
                                Ok(_) => CraftingState::NeedsInput,
                                // TODO: handle the waste products somehow
                                Err(_) => CraftingState::Overproduction,
                            }
                        }
                        None => match crafter
                            .output
                            .add_items_all_or_nothing(recipe.outputs(), &item_manifest)
                        {
                            Ok(()) => CraftingState::NeedsInput,
                            Err(_) => CraftingState::FullAndBlocked,
                        },
                    }
                } else {
                    CraftingState::NoRecipe
                }
            }
            CraftingState::FullAndBlocked => {
                let mut item_slots = crafter.output.iter();
                match item_slots.any(|slot| slot.is_full()) {
                    true => CraftingState::FullAndBlocked,
                    false => CraftingState::NeedsInput,
                }
            }
        };
    }
}

/// Sessile organisms gain energy when they finish crafting recipes.
fn gain_energy_when_crafting_completes(
    mut sessile_query: Query<(
        &mut EnergyPool,
        &mut Lifecycle,
        &CraftingState,
        &ActiveRecipe,
    )>,
    recipe_manifest: Res<RecipeManifest>,
) {
    for (mut energy_pool, mut lifecycle, crafting_state, active_recipe) in sessile_query.iter_mut()
    {
        if matches!(crafting_state, CraftingState::RecipeComplete) {
            if let Some(recipe_id) = active_recipe.recipe_id() {
                let recipe = recipe_manifest.get(*recipe_id);
                if let Some(energy) = recipe.energy() {
                    let proposed = energy_pool.current() + *energy;
                    energy_pool.set_current(proposed);
                    lifecycle.record_energy_gained(*energy);
                }
            }
        }
    }
}

/// Causes crafting structures to emit signals based on the items they have and need.
pub(crate) fn set_crafting_emitter(
    mut crafting_query: Query<(
        &mut Emitter,
        &InputInventory,
        &OutputInventory,
        &CraftingState,
        &Id<Structure>,
        &WorkersPresent,
        &ActiveRecipe,
    )>,
    recipe_manifest: Res<RecipeManifest>,
) {
    for (
        mut emitter,
        input_inventory,
        output_inventory,
        crafting_state,
        &structure_id,
        workers_present,
        active_recipe,
    ) in crafting_query.iter_mut()
    {
        // Reset and recompute all signals
        emitter.signals.clear();

        // Input signals
        for item_slot in input_inventory.iter() {
            if !item_slot.is_full() {
                let signal_type = SignalType::Pull(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            }
        }

        // Output signals
        for item_slot in output_inventory.iter() {
            if item_slot.is_full() {
                let signal_type = SignalType::Push(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            } else if !item_slot.is_empty() {
                let signal_type = SignalType::Contains(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            }
        }

        // Work signals
        if let CraftingState::InProgress { .. } = crafting_state {
            if let Some(recipe_id) = active_recipe.recipe_id() {
                let recipe = recipe_manifest.get(*recipe_id);
                if workers_present.needs_more() && recipe.needs_workers() {
                    let signal_strength = SignalStrength::new(100.);
                    emitter
                        .signals
                        .push((SignalType::Work(structure_id), signal_strength));
                }
            }
        }
    }
}

/// Causes storage structures to emit signals based on the items they have and accept.
pub(crate) fn set_storage_emitter(
    mut crafting_query: Query<(&mut Emitter, &StorageInventory)>,
    item_manifest: Res<ItemManifest>,
) {
    for (mut emitter, storage_inventory) in crafting_query.iter_mut() {
        // Reset and recompute all signals
        emitter.signals.clear();

        match storage_inventory.reserved_for() {
            // Item-specific storage
            Some(item_id) => {
                // If there's space, signal that
                if storage_inventory.remaining_space_for_item(item_id, &item_manifest) > 0 {
                    let signal_type = SignalType::Stores(item_id);
                    let signal_strength = SignalStrength::new(10.);
                    emitter.signals.push((signal_type, signal_strength));
                }

                // If there's any inventory, signal that
                if storage_inventory.item_count(item_id) > 0 {
                    let signal_type = SignalType::Contains(item_id);
                    let signal_strength = SignalStrength::new(10.);
                    emitter.signals.push((signal_type, signal_strength));
                }
            }
            // Junk drawer
            None => {
                // You could put anything in here!
                for item_id in item_manifest.variants() {
                    // If there's space, signal that
                    if storage_inventory.remaining_space_for_item(item_id, &item_manifest) > 0 {
                        let signal_type = SignalType::Stores(item_id);
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength));
                    }

                    // If there's any inventory, signal that
                    if storage_inventory.item_count(item_id) > 0 {
                        let signal_type = SignalType::Contains(item_id);
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength));
                    }
                }
            }
        }
    }
}

/// The space in storage inventories is not reserved
fn clear_empty_storage_slots(mut query: Query<&mut StorageInventory>) {
    for mut storage_inventory in query.iter_mut() {
        storage_inventory.clear_empty_slots();
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
            &'static Id<Structure>,
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
        structure_pos: TilePos,
        structure_id: Id<Structure>,
        map_geometry: &MapGeometry,
    ) -> Option<Entity> {
        // Prioritize ghosts over structures to allow for replacing structures by building
        let entity = if let Some(ghost_entity) = map_geometry.ghost_index.get(&structure_pos) {
            *ghost_entity
        } else {
            *map_geometry.structure_index.get(&structure_pos)?
        };

        let (found_crafting_state, found_structure_id, workers_present) =
            self.query.get(entity).ok()?;

        if *found_structure_id != structure_id {
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

/// Add crafting capabilities to structures.
pub(crate) struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Load this from an asset file
        let mut recipe_manifest: RecipeManifest = Manifest::new();
        recipe_manifest.insert(
            "acacia_leaf_production",
            RecipeData::acacia_leaf_production(),
        );
        recipe_manifest.insert(
            "leuco_chunk_production",
            RecipeData::leuco_chunk_production(),
        );
        recipe_manifest.insert("ant_egg_production", RecipeData::ant_egg_production());
        recipe_manifest.insert("hatch_ants", RecipeData::hatch_ants());

        app.insert_resource(recipe_manifest).add_systems(
            (
                progress_crafting,
                gain_energy_when_crafting_completes.after(progress_crafting),
                set_crafting_emitter.after(progress_crafting),
                set_storage_emitter,
                clear_empty_storage_slots,
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
