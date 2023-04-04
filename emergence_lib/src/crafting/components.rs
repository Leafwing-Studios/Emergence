//! The components needed to create a [`CraftingBundle`].

use super::{
    item_tags::ItemTag,
    recipe::{Recipe, RecipeData, RecipeInput, RecipeManifest, RecipeOutput},
};

use crate::{
    asset_management::manifest::Id,
    items::{
        errors::{AddManyItemsError, AddOneItemError},
        inventory::Inventory,
        item_manifest::{Item, ItemManifest},
        slot::ItemSlot,
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
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputInventory {
    /// Accepts precisely the provided inputs
    Exact {
        /// The required items per batch of this recipe.
        inventory: Inventory,
    },
    /// Accepts any input that has the correct tags
    Tagged {
        /// The required tag to use this recipe.
        tag: ItemTag,
        /// The currently stored items
        inventory: Inventory,
    },
}

impl InputInventory {
    /// Returns a reference the underlying [`Inventory`].
    pub fn inventory(&self) -> &Inventory {
        match self {
            InputInventory::Exact { inventory } => inventory,
            InputInventory::Tagged { inventory, .. } => inventory,
        }
    }

    /// Returns a mutable reference the underlying [`Inventory`].
    fn inventory_mut(&mut self) -> &mut Inventory {
        match self {
            InputInventory::Exact { inventory } => inventory,
            InputInventory::Tagged { inventory, .. } => inventory,
        }
    }

    /// Returns an iterator over the items currently in this inventory.
    pub fn iter(&self) -> impl Iterator<Item = &ItemSlot> {
        self.inventory().iter()
    }

    /// Returns the number of items in this inventory.
    pub fn len(&self) -> usize {
        // PERF: this is slow and lazy
        self.iter().count()
    }

    /// Is this inventory empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Does this inventory have space for at least one item of the given type?
    pub fn currently_accepts(&self, item_id: Id<Item>, item_manifest: &ItemManifest) -> bool {
        match self {
            InputInventory::Exact { inventory } => {
                inventory.remaining_reserved_space_for_item(item_id) > 0
            }
            InputInventory::Tagged { tag, inventory } => {
                item_manifest.has_tag(item_id, *tag)
                    && inventory.remaining_space_for_item(item_id, item_manifest) > 0
            }
        }
    }

    /// Try to add items to this inventory.
    pub fn fill_with_items(
        &mut self,
        item_count: &ItemCount,
        item_manifest: &ItemManifest,
    ) -> Result<(), AddToInputError> {
        if let InputInventory::Tagged { tag, .. } = self {
            if !item_manifest.has_tag(item_count.item_id, *tag) {
                return Err(AddToInputError::IncorrectItemTags);
            }
        };

        match self
            .inventory_mut()
            .add_item_all_or_nothing(item_count, item_manifest)
        {
            Ok(()) => Ok(()),
            Err(AddOneItemError { excess_count }) => {
                Err(AddToInputError::NotEnoughSpace { excess_count })
            }
        }
    }

    /// Try to remove the items specified by `recipe` from the inventory.
    pub fn consume_items(
        &mut self,
        recipe_input: &RecipeInput,
        item_manifest: &ItemManifest,
    ) -> Result<(), ConsumeInputError> {
        let inventory = self.inventory_mut();

        match recipe_input {
            RecipeInput::Exact(item_counts) => {
                match inventory.remove_items_all_or_nothing(item_counts) {
                    Ok(()) => Ok(()),
                    Err(_) => Err(ConsumeInputError::NotEnoughItems),
                }
            }
            RecipeInput::Flexible { tag, count } => {
                let mut remaining_to_remove = *count;
                let mut proposed_removal: Vec<ItemCount> = Vec::new();

                for item_slot in inventory.iter() {
                    // Verify that all items in the inventory are correct
                    if !item_manifest.has_tag(item_slot.item_id(), *tag) {
                        return Err(ConsumeInputError::IncorrectItemTags);
                    }

                    // Remove items from the inventory, beginning at the start of the inventory
                    let n = item_slot.count();
                    let removed_from_this_stack = std::cmp::min(n, remaining_to_remove);
                    proposed_removal.push(ItemCount {
                        item_id: item_slot.item_id(),
                        count: removed_from_this_stack,
                    });
                    remaining_to_remove -= removed_from_this_stack;

                    if remaining_to_remove == 0 {
                        break;
                    }
                }

                if remaining_to_remove > 0 {
                    return Err(ConsumeInputError::NotEnoughItems);
                }

                match inventory.remove_items_all_or_nothing(&proposed_removal) {
                    Ok(()) => Ok(()),
                    Err(_) => panic!("Inventory should have had enough items to remove"),
                }
            }
        }
    }

    /// Randomizes the contents of this inventory so that each slot is somewhere between empty and full.
    ///
    /// Note that this only works for [`InputInventory::Exact`].
    pub(super) fn randomize(&mut self, rng: &mut ThreadRng) {
        if let InputInventory::Exact { inventory } = self {
            for item_slot in inventory.iter_mut() {
                item_slot.randomize(rng);
            }
        }
    }

    /// The pretty formatting for this type.
    pub fn display(&self, item_manifest: &ItemManifest) -> String {
        match self {
            InputInventory::Exact { inventory } => inventory.display(item_manifest),
            InputInventory::Tagged { tag, inventory } => {
                format!("{}: {}", tag, inventory.display(item_manifest))
            }
        }
    }
}

/// An error that can occur when trying to consume items from an [`InputInventory`].
pub enum ConsumeInputError {
    /// Not enough items in the inventory.
    NotEnoughItems,
    /// The items in the inventory did not match the provided recipe.
    IncorrectItemTags,
}

/// An error that can occur when trying to add items to an [`InputInventory`].
pub enum AddToInputError {
    /// Not enough space in the inventory.
    NotEnoughSpace {
        /// The items that could not be added.
        excess_count: ItemCount,
    },
    /// The items in the inventory did not match the provided recipe.
    IncorrectItemTags,
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
                        quotient as u32
                    } else {
                        quotient as u32 + 1
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
                input_inventory: InputInventory::Exact {
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
