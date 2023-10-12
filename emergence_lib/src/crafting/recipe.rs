//! Instructions to craft items.

use crate::asset_management::manifest::loader::IsRawManifest;
use crate::asset_management::manifest::{Id, Manifest};
use crate::items::item_manifest::{Item, ItemManifest};
use crate::items::{inventory::Inventory, ItemCount};
use crate::light::shade::ReceivedLight;
use crate::light::Illuminance;
use crate::{
    crafting::inventories::{InputInventory, OutputInventory},
    organisms::energy::Energy,
};
use bevy::prelude::*;
use bevy::reflect::{Reflect, TypeUuid};
use bevy::utils::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};

use super::item_tags::ItemTag;

/// The marker type for [`Id<Recipe>`](super::Id).
#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
pub struct Recipe;

/// Stores the read-only definitions for all recipes.
pub type RecipeManifest = Manifest<Recipe, RecipeData>;

/// A recipe to turn a set of items into different items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeData {
    /// The inputs needed to craft the recipe.
    pub inputs: RecipeInput,

    /// The outputs generated by crafting.
    pub outputs: RecipeOutput,

    /// The time needed to craft the recipe.
    pub craft_time: Duration,

    /// The conditions that must be met to craft the recipe.
    pub conditions: RecipeConditions,

    /// The amount of [`Energy`] produced by making this recipe, if any.
    ///
    /// This is only relevant to living structures.
    pub energy: Option<Energy>,
}

/// The items needed to craft a recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecipeInput {
    /// The recipe requires exactly the provided number of each input.
    Exact(Vec<ItemCount>),
    /// The recipe requires a fixed number of inputs that meet the provided conditions.
    Flexible {
        /// The conditions that inputs must meet.
        tag: ItemTag,
        /// The number of inputs that must meet the tag.
        count: u32,
    },
}

impl RecipeInput {
    /// No inputs are needed.
    pub const EMPTY: RecipeInput = RecipeInput::Exact(Vec::new());

    /// The number of slots needed to craft this recipe.
    pub fn len(&self) -> usize {
        match self {
            Self::Exact(inputs) => inputs.len(),
            Self::Flexible { .. } => 1,
        }
    }

    /// Is anything needed to craft this recipe?
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Exact(inputs) => inputs.is_empty(),
            Self::Flexible { .. } => false,
        }
    }
}

/// The unprocessed equivalent of [`RecipeInput`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RawRecipeInput {
    /// The recipe requires exactly the provided number of each input.
    Exact(HashMap<String, u32>),
    /// The recipe requires a fixed number of inputs that meet the provided conditions.
    Flexible {
        /// The conditions that inputs must meet.
        tag: ItemTag,
        /// The number of inputs that must meet the tag.
        count: u32,
    },
}

impl RawRecipeInput {
    /// No inputs are needed.
    pub fn empty() -> Self {
        Self::Exact(HashMap::new())
    }

    /// Exactly one input is needed.
    pub fn single(item_name: &str, count: u32) -> Self {
        let mut inputs = HashMap::new();
        inputs.insert(item_name.to_string(), count);
        Self::Exact(inputs)
    }
}

impl From<RawRecipeInput> for RecipeInput {
    fn from(raw_input: RawRecipeInput) -> Self {
        match raw_input {
            RawRecipeInput::Exact(raw_data) => Self::Exact(
                raw_data
                    .into_iter()
                    .map(|(item_name, count)| ItemCount {
                        item_id: Id::from_name(item_name),
                        count,
                    })
                    .collect(),
            ),
            RawRecipeInput::Flexible { tag, count } => Self::Flexible { tag, count },
        }
    }
}

/// The items produced by a recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecipeOutput {
    /// The recipe produces a fixed number of each output.
    Deterministic(Vec<ItemCount>),
    /// The recipe produces a random number of each output.
    Stochastic(Vec<(Id<Item>, f32)>),
}

impl RecipeOutput {
    /// Nothing is produced.
    pub const EMPTY: RecipeOutput = RecipeOutput::Deterministic(Vec::new());

    /// Convert the raw data into a [`RecipeOutput`].
    ///
    /// If all the counts are integers, then the recipe is deterministic.
    /// Otherwise, it is stochastic.
    fn from_raw(raw_data: HashMap<String, f32>) -> Self {
        let all_integers = raw_data.values().all(|count| count.fract() == 0.0);

        match all_integers {
            true => Self::Deterministic(
                raw_data
                    .into_iter()
                    .map(|(item_name, count)| ItemCount {
                        item_id: Id::from_name(item_name),
                        count: count as u32,
                    })
                    .collect(),
            ),
            false => Self::Stochastic(
                raw_data
                    .into_iter()
                    .map(|(item_name, count)| (Id::from_name(item_name), count))
                    .collect(),
            ),
        }
    }

    /// The number of outputs produced by this recipe.
    pub fn len(&self) -> usize {
        match self {
            Self::Deterministic(outputs) => outputs.len(),
            Self::Stochastic(outputs) => outputs.len(),
        }
    }

    /// Are there any outputs produced by this recipe?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The [`Id<Item>`]s of the items produced by this recipe.
    pub fn item_ids(&self) -> Vec<Id<Item>> {
        match self {
            Self::Deterministic(outputs) => outputs.iter().map(|output| output.item_id).collect(),
            Self::Stochastic(outputs) => outputs.iter().map(|(item_id, _)| *item_id).collect(),
        }
    }
}

/// The unprocessed equivalent of [`RecipeData`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawRecipeData {
    /// The inputs needed to craft the recipe.
    pub inputs: RawRecipeInput,

    /// The outputs generated by crafting.
    pub outputs: HashMap<String, f32>,

    /// The time needed to craft the recipe.
    pub craft_time: f32,

    /// The conditions that must be met to craft the recipe.
    pub conditions: Option<RecipeConditions>,

    /// The amount of [`Energy`] produced by making this recipe, if any.
    ///
    /// This is only relevant to living structures.
    pub energy: Option<Energy>,
}

impl From<RawRecipeData> for RecipeData {
    fn from(raw: RawRecipeData) -> Self {
        Self {
            inputs: raw.inputs.into(),
            outputs: RecipeOutput::from_raw(raw.outputs),
            craft_time: Duration::from_secs_f32(raw.craft_time),
            conditions: raw.conditions.unwrap_or_default(),
            energy: raw.energy,
        }
    }
}

impl RecipeData {
    /// Are the conditions to craft this recipe met?
    pub(crate) fn satisfied(&self, workers: u8, received_light: &ReceivedLight) -> bool {
        self.conditions.satisfied(workers, received_light)
    }

    /// An inventory with empty slots for all of the inputs of this recipe.
    pub(crate) fn input_inventory(&self, item_manifest: &ItemManifest) -> InputInventory {
        match self.inputs {
            RecipeInput::Exact(ref inputs) => {
                let mut inventory = Inventory::new(self.inputs.len(), None);

                for item_count in inputs.iter() {
                    inventory.add_empty_slot(item_count.item_id, item_manifest);
                }

                InputInventory::Exact { inventory }
            }
            RecipeInput::Flexible { tag, .. } => InputInventory::Tagged {
                tag,
                inventory: Inventory::new(1, None),
            },
        }
    }

    /// An inventory with empty slots for all of the outputs of this recipe.
    pub(crate) fn output_inventory(&self, item_manifest: &ItemManifest) -> OutputInventory {
        let mut inventory = Inventory::new(self.outputs.len(), None);
        for item_id in self.outputs.item_ids() {
            inventory.add_empty_slot(item_id, item_manifest);
        }
        OutputInventory { inventory }
    }

    /// The number of workers this recipe needs to be crafted at all.
    pub(crate) fn workers_required(&self) -> u8 {
        self.conditions.workers_required
    }

    /// Does this recipe need workers to produce?
    pub(crate) fn needs_workers(&self) -> bool {
        self.conditions.workers_required > 0
    }

    /// The pretty formatting of this type
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        let input_str: String = match self.inputs {
            RecipeInput::Exact(ref inputs) => inputs
                .iter()
                .map(|input| input.display(item_manifest))
                .join(", "),
            RecipeInput::Flexible { tag, count } => format!("{count}x {tag}"),
        };

        let output_strings: Vec<String> = self
            .outputs
            .item_ids()
            .iter()
            .map(|output_id| item_manifest.name(*output_id).to_string())
            .collect();
        let output_str = output_strings.join(", ");

        let duration_str = format!("{:.2}", self.craft_time.as_secs_f32());

        let condition_str = if self.conditions == RecipeConditions::default() {
            String::new()
        } else {
            format!("\nwhen {}", self.conditions)
        };

        format!("[{input_str}] -> [{output_str}] | {duration_str} s{condition_str}")
    }
}

/// The environmental conditions needed for work to be done on a recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RecipeConditions {
    /// The number of workers required to advance this recipe.
    pub workers_required: u8,
    /// The range of light levels that are acceptable for this recipe.
    pub allowable_light_range: Option<Threshold<Illuminance>>,
}

impl Display for RecipeConditions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.workers_required > 0 {
            write!(f, "Workers: {}", self.workers_required)?;
        }
        if let Some(range) = &self.allowable_light_range {
            write!(f, "Light: {}", *range)?;
        }
        Ok(())
    }
}

impl RecipeConditions {
    /// No conditions are required to craft this recipe.
    pub const NONE: RecipeConditions = RecipeConditions {
        workers_required: 0,
        allowable_light_range: None,
    };

    /// Creates a new [`RecipeConditions`].
    pub const fn new(workers_required: u8, allowable_light_range: Threshold<Illuminance>) -> Self {
        Self {
            workers_required,
            allowable_light_range: Some(allowable_light_range),
        }
    }

    /// Are the conditions to craft this recipe met?
    fn satisfied(&self, workers: u8, received_light: &ReceivedLight) -> bool {
        let work_satisfied = self.workers_required == 0 || workers >= self.workers_required;
        let light_satisfied = self
            .allowable_light_range
            .as_ref()
            .map_or(true, |range| range.contains(received_light.0));

        work_satisfied && light_satisfied
    }
}

/// A viable range of a value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Threshold<T: PartialOrd> {
    /// The minimum value of the range.
    min: T,
    /// The maximum value of the range.
    max: T,
}

impl<T: PartialOrd> Threshold<T> {
    /// Creates a new [`Threshold`].
    pub fn new(min: T, max: T) -> Self {
        assert!(min <= max);

        Self { min, max }
    }

    /// Returns true if the value is within the threshold.
    fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }
}

impl<T: Display + PartialOrd> Display for Threshold<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.min, self.max)
    }
}

/// The [`RecipeManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "c711b30c-c3ff-4b86-92d0-f1aff2ec7818"]
pub struct RawRecipeManifest {
    /// The data for each item.
    pub recipes: HashMap<String, RawRecipeData>,
}

impl IsRawManifest for RawRecipeManifest {
    const EXTENSION: &'static str = "recipe_manifest.json";

    type Marker = Recipe;
    type Data = RecipeData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.recipes.clone() {
            let data = raw_data.into();

            manifest.insert(raw_id, data)
        }

        manifest
    }
}

/// The recipe that is currently being crafted, if any.
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ActiveRecipe(pub(super) Option<Id<Recipe>>);

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
