//! Instructions to craft items.

use std::{fmt::Display, time::Duration};

use crate::asset_management::manifest::Manifest;

use super::{count::ItemCount, ItemId};

/// The unique identifier of a recipe.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct RecipeId(&'static str);

impl RecipeId {
    /// The ID of the recipe for the leaf production of acacia plants.
    pub fn acacia_leaf_production() -> Self {
        Self("acacia_leaf_production")
    }
}

impl Display for RecipeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A recipe to turn a set of items into different items.
#[derive(Debug, Clone)]
pub struct Recipe {
    /// The inputs needed to craft the recipe.
    inputs: Vec<ItemCount>,

    /// The outputs generated by crafting.
    outputs: Vec<ItemCount>,

    /// The time needed to craft the recipe.
    craft_time: Duration,
}

impl Recipe {
    /// Create a new recipe with the given inputs, outputs and craft time.
    pub fn new(inputs: Vec<ItemCount>, outputs: Vec<ItemCount>, craft_time: Duration) -> Self {
        Self {
            inputs,
            outputs,
            craft_time,
        }
    }

    // TODO: Remove this once we load recipes from asset files
    /// An acacia plant producing leaves.
    pub fn acacia_leaf_production() -> Self {
        Recipe::new(
            Vec::new(),
            vec![ItemCount::one(ItemId::acacia_leaf())],
            Duration::from_secs(10),
        )
    }

    /// The inputs needed to craft the recipe.
    pub fn inputs(&self) -> &Vec<ItemCount> {
        &self.inputs
    }

    /// The outputs generated by crafting.
    pub fn outputs(&self) -> &Vec<ItemCount> {
        &self.outputs
    }

    /// The time needed to craft the recipe.
    pub fn craft_time(&self) -> &Duration {
        &self.craft_time
    }
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input_strings: Vec<String> =
            self.inputs.iter().map(|input| format!("{input}")).collect();
        let input_str = input_strings.join(", ");

        let output_strings: Vec<String> = self
            .outputs
            .iter()
            .map(|output| format!("{output}"))
            .collect();
        let output_str = output_strings.join(", ");

        let duration_str = format!("{:.2}", self.craft_time().as_secs_f32());

        write!(f, "[{input_str}] -> [{output_str}] | {duration_str}s")
    }
}

/// The definitions for all recipes.
pub type RecipeManifest = Manifest<RecipeId, Recipe>;

#[cfg(test)]
mod tests {
    use crate::items::ItemId;

    use super::*;

    #[test]
    fn should_display_inputs_outputs_craft_time() {
        let recipe = Recipe {
            inputs: Vec::new(),
            outputs: vec![ItemCount::one(ItemId::acacia_leaf())],
            craft_time: Duration::from_secs(1),
        };

        assert_eq!(format!("{recipe}"), "[] -> [acacia_leaf (1)] | 1.00s")
    }
}
