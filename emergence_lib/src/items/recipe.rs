//! Instructions to craft items.

use super::{inventory::Inventory, ItemCount};
use crate::asset_management::manifest::Manifest;
use crate::{
    asset_management::manifest::{Id, ItemManifest},
    organisms::energy::Energy,
    simulation::light::{Illuminance, TotalLight},
    structures::crafting::{InputInventory, OutputInventory},
};
use bevy::reflect::{FromReflect, Reflect};
use std::{fmt::Display, time::Duration};

/// The marker type for [`Id<Recipe>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Recipe;

/// Stores the read-only definitions for all recipes.
pub(crate) type RecipeManifest = Manifest<Recipe, RecipeData>;

/// A recipe to turn a set of items into different items.
#[derive(Debug, Clone)]
pub(crate) struct RecipeData {
    /// The inputs needed to craft the recipe.
    inputs: Vec<ItemCount>,

    /// The outputs generated by crafting.
    outputs: Vec<ItemCount>,

    /// The time needed to craft the recipe.
    craft_time: Duration,

    /// The conditions that must be met to craft the recipe.
    conditions: RecipeConditions,

    /// The amount of [`Energy`] produced by making this recipe, if any.
    ///
    /// This is only relevant to living structures.
    energy: Option<Energy>,
}

impl RecipeData {
    /// Create a new recipe with the given inputs, outputs and craft time.
    pub(crate) fn new(
        inputs: Vec<ItemCount>,
        outputs: Vec<ItemCount>,
        craft_time: Duration,
        conditions: RecipeConditions,
        energy: Option<Energy>,
    ) -> Self {
        Self {
            inputs,
            outputs,
            craft_time,
            conditions,
            energy,
        }
    }

    /// The inputs needed to craft the recipe.
    pub(crate) fn inputs(&self) -> &Vec<ItemCount> {
        &self.inputs
    }

    /// The outputs generated by crafting.
    pub(crate) fn outputs(&self) -> &Vec<ItemCount> {
        &self.outputs
    }

    /// The time needed to craft the recipe.
    pub(crate) fn craft_time(&self) -> Duration {
        self.craft_time
    }

    /// Are the conditions to craft this recipe met?
    pub(crate) fn satisfied(&self, workers: u8, total_light: &TotalLight) -> bool {
        self.conditions.satisfied(workers, total_light)
    }

    /// An inventory with empty slots for all of the inputs of this recipe.
    pub(crate) fn input_inventory(&self, item_manifest: &ItemManifest) -> InputInventory {
        let mut inventory = Inventory::new(self.inputs.len(), None);
        for item_count in &self.inputs {
            inventory.add_empty_slot(item_count.item_id, item_manifest);
        }
        InputInventory { inventory }
    }

    /// An inventory with empty slots for all of the outputs of this recipe.
    pub(crate) fn output_inventory(&self, item_manifest: &ItemManifest) -> OutputInventory {
        let mut inventory = Inventory::new(self.outputs.len(), None);
        for item_count in &self.outputs {
            inventory.add_empty_slot(item_count.item_id, item_manifest);
        }
        OutputInventory { inventory }
    }

    /// The amount of energy produced by crafting the recipe, if any.
    pub(crate) fn energy(&self) -> &Option<Energy> {
        &self.energy
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
        let input_strings: Vec<String> = self
            .inputs
            .iter()
            .map(|input| input.display(item_manifest))
            .collect();
        let input_str = input_strings.join(", ");

        let output_strings: Vec<String> = self
            .outputs
            .iter()
            .map(|output| output.display(item_manifest))
            .collect();
        let output_str = output_strings.join(", ");

        let duration_str = format!("{:.2}", self.craft_time().as_secs_f32());

        let condition_str = if self.conditions == RecipeConditions::NONE {
            String::new()
        } else {
            format!("\nwhen {}", self.conditions)
        };

        format!("[{input_str}] -> [{output_str}] | {duration_str} s{condition_str}")
    }
}

// TODO: Remove this once we load recipes from asset files
impl RecipeData {
    /// An acacia plant producing leaves.
    pub(crate) fn acacia_leaf_production() -> Self {
        RecipeData::new(
            Vec::new(),
            vec![ItemCount::one(Id::from_name("acacia_leaf"))],
            Duration::from_secs(3),
            RecipeConditions::new(0, Threshold::new(Illuminance(5e3), Illuminance(6e4))),
            Some(Energy(20.)),
        )
    }

    /// A leuco mushroom processing acacia leaves
    pub(crate) fn leuco_chunk_production() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::from_name("acacia_leaf"))],
            vec![ItemCount::one(Id::from_name("leuco_chunk"))],
            Duration::from_secs(2),
            RecipeConditions::NONE,
            Some(Energy(40.)),
        )
    }

    /// An ant hive producing eggs.
    pub(crate) fn ant_egg_production() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::from_name("leuco_chunk"))],
            vec![ItemCount::one(Id::from_name("ant_egg"))],
            Duration::from_secs(5),
            RecipeConditions {
                workers_required: 2,
                allowable_light_range: None,
            },
            None,
        )
    }

    /// An ant hive producing eggs.
    pub(crate) fn hatch_ants() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::from_name("ant_egg"))],
            vec![],
            Duration::from_secs(10),
            RecipeConditions {
                workers_required: 1,
                allowable_light_range: None,
            },
            None,
        )
    }
}

/// The environmental conditions needed for work to be done on a recipe.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecipeConditions {
    /// The number of workers required to advance this recipe.
    workers_required: u8,
    /// The range of light levels that are acceptable for this recipe.
    allowable_light_range: Option<Threshold<Illuminance>>,
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
    /// No special conditions are needed for this recipe.
    pub(crate) const NONE: RecipeConditions = RecipeConditions {
        workers_required: 0,
        allowable_light_range: None,
    };

    /// Creates a new [`RecipeConditions`].
    pub(crate) const fn new(
        workers_required: u8,
        allowable_light_range: Threshold<Illuminance>,
    ) -> Self {
        Self {
            workers_required,
            allowable_light_range: Some(allowable_light_range),
        }
    }

    /// Are the conditions to craft this recipe met?
    fn satisfied(&self, workers: u8, total_light: &TotalLight) -> bool {
        let work_satisfied = self.workers_required == 0 || workers >= self.workers_required;
        let light_satisfied = self
            .allowable_light_range
            .as_ref()
            .map_or(true, |range| range.contains(total_light.illuminance()));

        work_satisfied && light_satisfied
    }
}

/// A viable range of a value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Threshold<T: PartialOrd> {
    /// The minimum value of the range.
    min: T,
    /// The maximum value of the range.
    max: T,
}

impl<T: PartialOrd> Threshold<T> {
    /// Creates a new [`Threshold`].
    pub(crate) fn new(min: T, max: T) -> Self {
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
