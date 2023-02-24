//! Everything needed to make structures able to craft things.

use std::{fmt::Display, time::Duration};

use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashMap};
use leafwing_abilities::prelude::Pool;

use crate::{
    items::{
        inventory::{Inventory, InventoryState},
        recipe::{Recipe, RecipeId, RecipeManifest},
        ItemData, ItemId, ItemManifest,
    },
    organisms::{energy::EnergyPool, Organism},
    signals::{Emitter, SignalStrength, SignalType},
};

/// The current state in the crafting progress.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum CraftingState {
    /// There are resources missing for the recipe.
    #[default]
    NeedsInput,
    /// The resource cost has been paid and the recipe is being crafted.
    InProgress,
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
        let str = match self {
            CraftingState::NeedsInput => "Waiting for input",
            CraftingState::InProgress => "In progress",
            CraftingState::RecipeComplete => "Recipe complete",
            CraftingState::FullAndBlocked => "Blocked",
            CraftingState::Overproduction => "Overproduction",
            CraftingState::NoRecipe => "No recipe set",
        };

        write!(f, "{str}")
    }
}

/// The input inventory for a structure.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub(crate) struct InputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

/// The output inventory for a structure.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct OutputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

/// The recipe that is currently being crafted, if any.
#[derive(Component, Debug, Default)]
pub(crate) struct ActiveRecipe(Option<RecipeId>);

impl ActiveRecipe {
    /// The ID of the currently active recipe, if one has been selected.
    pub(crate) fn recipe_id(&self) -> &Option<RecipeId> {
        &self.0
    }
}

/// The time remaining until the recipe has been crafted.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct CraftTimer(Timer);

impl CraftTimer {
    /// The timer indicating how much longer the crafting process will take.
    pub(crate) fn timer(&self) -> &Timer {
        &self.0
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

    /// The "cooldown" for crafting.
    craft_timer: CraftTimer,

    /// The current state for the crafting process.
    craft_state: CraftingState,

    /// Emits signals, drawing units towards this structure to ensure crafting flows smoothly
    emitter: Emitter,
}

impl CraftingBundle {
    /// Create a new crafting bundle without an active recipe set.
    pub(crate) fn new(
        starting_recipe: Option<RecipeId>,
        recipe_manifest: &RecipeManifest,
        item_manifest: &ItemManifest,
    ) -> Self {
        if let Some(recipe_id) = starting_recipe {
            let recipe = recipe_manifest.get(recipe_id);

            Self {
                input_inventory: InputInventory {
                    inventory: recipe.input_inventory(item_manifest),
                },
                output_inventory: OutputInventory {
                    inventory: recipe.output_inventory(item_manifest),
                },
                craft_timer: CraftTimer(Timer::new(Duration::default(), TimerMode::Once)),
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::NeedsInput,
                emitter: Emitter::default(),
            }
        } else {
            Self {
                input_inventory: InputInventory {
                    inventory: Inventory::new(0),
                },
                output_inventory: OutputInventory {
                    inventory: Inventory::new(1),
                },
                craft_timer: CraftTimer(Timer::new(Duration::ZERO, TimerMode::Once)),
                active_recipe: ActiveRecipe(None),
                craft_state: CraftingState::NeedsInput,
                emitter: Emitter::default(),
            }
        }
    }
}

/// Data needed for [`progress_crafting`].
#[derive(WorldQuery)]
#[world_query(mutable)]
struct CraftingQuery {
    /// The recipe of the crafter
    active_recipe: &'static ActiveRecipe,
    /// The time remaining to complete the recipe
    timer: &'static mut CraftTimer,
    /// The status of crafting
    state: &'static mut CraftingState,
    /// The inputs
    input: &'static mut InputInventory,
    /// The outputs
    output: &'static mut OutputInventory,
    /// Is this an organism?
    maybe_organism: Option<&'static Organism>,
}

/// Progress the state of recipes that are being crafted.
fn progress_crafting(
    time: Res<Time>,
    recipe_manifest: Res<RecipeManifest>,
    item_manifest: Res<ItemManifest>,
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
                        Ok(()) => {
                            crafter.timer.set_duration(recipe.craft_time());
                            crafter.timer.reset();
                            CraftingState::InProgress
                        }
                        Err(_) => CraftingState::NeedsInput,
                    }
                } else {
                    CraftingState::NoRecipe
                }
            }
            CraftingState::InProgress => {
                crafter.timer.tick(time.delta());

                if crafter.timer.finished() {
                    CraftingState::RecipeComplete
                } else {
                    CraftingState::InProgress
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
    mut sessile_query: Query<(&mut EnergyPool, &CraftingState, &ActiveRecipe)>,
    recipe_manifest: Res<RecipeManifest>,
) {
    for (mut energy_pool, crafting_state, active_recipe) in sessile_query.iter_mut() {
        if matches!(crafting_state, CraftingState::RecipeComplete) {
            if let Some(recipe_id) = active_recipe.recipe_id() {
                let recipe = recipe_manifest.get(*recipe_id);
                if let Some(energy) = recipe.energy() {
                    let proposed = energy_pool.current() + *energy;
                    energy_pool.set_current(proposed);
                }
            }
        }
    }
}

/// Causes crafting structures to emit signals based on the items they have and need.
// TODO: change neglect based on inventory fullness and structure energy level
fn set_emitter(mut crafting_query: Query<(&mut Emitter, &InputInventory, &OutputInventory)>) {
    use InventoryState::*;

    /// The rate at which neglect rises and falls for crafting structures.
    ///
    /// Should be positive
    const NEGLECT_RATE: f32 = 0.05;
    /// The minimum neglect that a crafting structure can have.
    ///
    /// This ensures that buildings are not neglected forever after being satisfied for a while.
    const MIN_NEGLECT: f32 = 0.05;

    for (mut emitter, input_inventory, output_inventory) in crafting_query.iter_mut() {
        // Reset and recompute all signals
        emitter.signals.clear();

        for item_slot in input_inventory.iter() {
            if !item_slot.is_full() {
                let signal_type = SignalType::Pull(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            }
        }

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

        let input_inventory_state = input_inventory.state();
        let output_inventory_state = output_inventory.state();

        let delta_neglect = match (input_inventory_state, output_inventory_state) {
            // Needs more inputs
            (Empty, _) => NEGLECT_RATE,
            // Working happily
            (Partial, Empty | Partial) => -NEGLECT_RATE,
            // Outputs should be removed
            (_, Full) => NEGLECT_RATE,
            // Waiting to craft
            (Full, Empty | Partial) => -NEGLECT_RATE,
        };

        emitter.neglect_multiplier = (emitter.neglect_multiplier + delta_neglect).max(MIN_NEGLECT);
    }
}

/// Add crafting capabilities to structures.
pub(crate) struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Load this from an asset file
        let mut item_manifest = HashMap::new();
        item_manifest.insert(ItemId::acacia_leaf(), ItemData::acacia_leaf());
        item_manifest.insert(ItemId::leuco_chunk(), ItemData::leuco_chunk());

        // TODO: Load this from an asset file
        let mut recipe_manifest = HashMap::new();
        recipe_manifest.insert(
            RecipeId::acacia_leaf_production(),
            Recipe::acacia_leaf_production(),
        );
        recipe_manifest.insert(
            RecipeId::leuco_chunk_production(),
            Recipe::leuco_chunk_production(),
        );
        recipe_manifest.insert(RecipeId::ant_egg_production(), Recipe::ant_egg_production());

        app.insert_resource(ItemManifest::new(item_manifest))
            .insert_resource(RecipeManifest::new(recipe_manifest))
            .add_system(progress_crafting)
            .add_system(gain_energy_when_crafting_completes.after(progress_crafting))
            .add_system(set_emitter.after(progress_crafting));
    }
}
