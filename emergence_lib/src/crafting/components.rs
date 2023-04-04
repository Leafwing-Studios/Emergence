//! The components needed to create a [`CraftingBundle`].

use super::recipe::{Recipe, RecipeData, RecipeManifest, RecipeOutput};

use crate::{
    asset_management::manifest::Id,
    items::{
        errors::{AddManyItemsError, AddOneItemError},
        inventory::Inventory,
        item_manifest::{Item, ItemManifest},
        ItemCount,
    },
    signals::Emitter,
    structures::structure_manifest::{Structure, StructureManifest},
};

use std::{fmt::Display, time::Duration};

use bevy::prelude::*;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng};
use serde::{Deserialize, Serialize};

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
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, PartialEq, Serialize, Deserialize)]
pub struct InputInventory {
    /// Inner storage
    pub inventory: Inventory,
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

    /// Produces the items specified by `recipe` and adds them to the inventory.
    pub(super) fn craft(
        &mut self,
        recipe: &RecipeData,
        item_manifest: &ItemManifest,
        rng: &mut ThreadRng,
    ) -> Result<(), AddManyItemsError> {
        let mut overflow: Vec<ItemCount> = Vec::new();

        match &recipe.outputs {
            RecipeOutput::Deterministic(outputs) => {
                for output in outputs {
                    let result = self.try_add_item(output, item_manifest);
                    if let Err(AddOneItemError { excess_count }) = result {
                        overflow.push(excess_count);
                    }
                }
            }
            RecipeOutput::Stochastic(outputs) => {
                let distribution = Uniform::new(0.0, 1.0);
                for (item_id, number) in outputs {
                    // Always produce items equal to quotient,
                    // and then produce one extra items with probability remainder.
                    let (quotient, remainder) = (number / 1.0, number % 1.0);
                    let count = if remainder == 0. || distribution.sample(rng) > remainder {
                        quotient as usize
                    } else {
                        quotient as usize + 1
                    };

                    let output = ItemCount::new(*item_id, count);

                    let result = self.try_add_item(&output, item_manifest);
                    if let Err(AddOneItemError { excess_count }) = result {
                        overflow.push(excess_count);
                    }
                }
            }
        };

        if overflow.is_empty() {
            Ok(())
        } else {
            Err(AddManyItemsError {
                excess_counts: overflow,
            })
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
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ActiveRecipe(Option<Id<Recipe>>);

/// The raw version of [`ActiveRecipe`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawActiveRecipe(Option<String>);

impl RawActiveRecipe {
    /// Creates a new [`RawActiveRecipe`], set to `recipe_name`.
    pub fn new(recipe_name: &str) -> Self {
        RawActiveRecipe(Some(recipe_name.to_string()))
    }
}

impl From<RawActiveRecipe> for ActiveRecipe {
    fn from(raw: RawActiveRecipe) -> Self {
        ActiveRecipe(raw.0.map(Id::from_name))
    }
}

impl ActiveRecipe {
    /// The un-set [`ActiveRecipe`].
    pub const NONE: ActiveRecipe = ActiveRecipe(None);

    /// Creates a new [`ActiveRecipe`], set to `recipe_id`
    pub fn new(recipe_id: Id<Recipe>) -> Self {
        ActiveRecipe(Some(recipe_id))
    }

    /// The ID of the currently active recipe, if one has been selected.
    pub fn recipe_id(&self) -> &Option<Id<Recipe>> {
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

/// The number of workers present / allowed at this structure.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
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

            let distribution = Uniform::new(Duration::ZERO, recipe.craft_time);
            let progress = distribution.sample(rng);
            let max_workers = structure_manifest.get(structure_id).max_workers;

            Self {
                input_inventory,
                output_inventory,
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::InProgress {
                    progress,
                    required: recipe.craft_time,
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
