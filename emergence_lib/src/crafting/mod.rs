//! Crafting and recipes.

use leafwing_abilities::prelude::Pool;
use recipe::{RawRecipeManifest, RecipeManifest};

use crate::{
    asset_management::manifest::{plugin::ManifestPlugin, Id},
    items::item_manifest::{ItemManifest, RawItemManifest},
    organisms::{energy::EnergyPool, lifecycle::Lifecycle, Organism},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::{light::TotalLight, SimulationSet},
    structures::structure_manifest::Structure,
};

use std::time::Duration;

use bevy::{ecs::query::WorldQuery, prelude::*};

use self::{
    components::{
        ActiveRecipe, CraftingState, InputInventory, OutputInventory, StorageInventory,
        WorkersPresent,
    },
    item_tags::ItemKind,
};

pub mod components;
pub mod item_tags;
pub mod recipe;

/// Add crafting capabilities to structures.
pub(crate) struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawItemManifest>::new())
            .add_plugin(ManifestPlugin::<RawRecipeManifest>::new())
            .add_systems(
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
    let rng = &mut rand::thread_rng();

    for mut crafter in crafting_query.iter_mut() {
        *crafter.state = match *crafter.state {
            CraftingState::NoRecipe => match crafter.active_recipe.recipe_id() {
                Some(_) => CraftingState::NeedsInput,
                None => CraftingState::NoRecipe,
            },
            CraftingState::NeedsInput | CraftingState::Overproduction => {
                if let Some(recipe_id) = crafter.active_recipe.recipe_id() {
                    let recipe = recipe_manifest.get(*recipe_id);
                    match crafter.input.consume_items(&recipe.inputs, &item_manifest) {
                        Ok(()) => CraftingState::InProgress {
                            progress: Duration::ZERO,
                            required: recipe.craft_time,
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
                        // Many hands make light work!
                        if recipe.workers_required() > 0 {
                            let work_ratio = crafter.workers_present.current() as f32
                                / recipe.workers_required() as f32;
                            updated_progress +=
                                Duration::from_secs_f32(time.period.as_secs_f32() * work_ratio);
                        } else {
                            updated_progress += time.period;
                        }

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
                            match crafter.output.craft(recipe, &item_manifest, rng) {
                                Ok(_) => CraftingState::NeedsInput,
                                // TODO: handle the waste products somehow
                                Err(_) => CraftingState::Overproduction,
                            }
                        }
                        None => match crafter.output.craft(recipe, &item_manifest, rng) {
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
                if let Some(energy) = recipe.energy {
                    let proposed = energy_pool.current() + energy;
                    energy_pool.set_current(proposed);
                    lifecycle.record_energy_gained(energy);
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
        match input_inventory {
            InputInventory::Exact { inventory } => {
                for item_slot in inventory.iter() {
                    if !item_slot.is_full() {
                        let signal_type = SignalType::Pull(ItemKind::Single(item_slot.item_id()));
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength));
                    }
                }
            }
            InputInventory::Tagged { tag, inventory } => {
                if !inventory.is_full() {
                    let signal_type = SignalType::Pull(ItemKind::Tag(*tag));
                    let signal_strength = SignalStrength::new(10.);
                    emitter.signals.push((signal_type, signal_strength));
                }
            }
        }

        // Output signals
        for item_slot in output_inventory.iter() {
            if item_slot.is_full() {
                let signal_type = SignalType::Push(ItemKind::Single(item_slot.item_id()));
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            } else if !item_slot.is_empty() {
                let signal_type = SignalType::Contains(ItemKind::Single(item_slot.item_id()));
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
                    let signal_type = SignalType::Stores(ItemKind::Single(item_id));
                    let signal_strength = SignalStrength::new(10.);
                    emitter.signals.push((signal_type, signal_strength));
                }

                // If there's any inventory, signal that
                if storage_inventory.item_count(item_id) > 0 {
                    let signal_type = SignalType::Contains(ItemKind::Single(item_id));
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
                        let signal_type = SignalType::Stores(ItemKind::Single(item_id));
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength));
                    }

                    // If there's any inventory, signal that
                    if storage_inventory.item_count(item_id) > 0 {
                        let signal_type = SignalType::Contains(ItemKind::Single(item_id));
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
