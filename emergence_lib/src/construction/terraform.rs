//! Tools to alter the terrain type and height.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    crafting::recipe::{RecipeConditions, RecipeData, RecipeInput, RecipeOutput},
    items::{item_manifest::Item, ItemCount},
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

/// An option presented to players for how to terraform the world.
///
/// These are generally higher level than the actual [`TerraformingAction`]s,
/// which represent the actual changes to the terrain that can be performed by units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingTool {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided [`Id<Terrain>`].
    Change(Id<Terrain>),
}

/// When `Zoning` is set, this is added  as a component added to terrain ghosts, causing them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TerraformingAction {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Set the desired terrain material of this tile
    Change(Id<Terrain>),
}

impl TerraformingAction {
    /// The recipe data for this action.
    pub(crate) fn recipe(&self) -> RecipeData {
        // All of this construction is so tightly coupled to the core game mechanics
        // that it seems very hard to store this in a data-driven way using the recipe manifest.
        // Fundmantally, terraforming doesn't seem to work the same way as other crafting recipes.
        let soil_id = Id::<Item>::from_name("soil".to_string());

        // TODO: vary these inventories based on the terrain type
        let input_counts = vec![ItemCount::new(soil_id, 10)];
        let ouput_counts = vec![ItemCount::new(soil_id, 10)];

        let inputs = match self {
            TerraformingAction::Raise => RecipeInput::Exact(input_counts),
            TerraformingAction::Lower => RecipeInput::EMPTY,
            TerraformingAction::Change(terrain) => RecipeInput::Exact(input_counts),
        };

        let outputs = match self {
            TerraformingAction::Raise => RecipeOutput::EMPTY,
            TerraformingAction::Lower => RecipeOutput::Deterministic(ouput_counts),
            TerraformingAction::Change(terrain) => RecipeOutput::Deterministic(ouput_counts),
        };

        RecipeData {
            inputs,
            outputs,
            craft_time: Duration::ZERO,
            conditions: RecipeConditions::NONE,
            energy: None,
        }
    }

    /// The pretty formatted name of this action.
    pub(crate) fn display(&self, terrain_manifest: &TerrainManifest) -> String {
        match self {
            Self::Raise => "Raise".to_string(),
            Self::Lower => "Lower".to_string(),
            Self::Change(terrain_id) => terrain_manifest.name(*terrain_id).to_string(),
        }
    }
}

impl From<TerraformingTool> for TerraformingAction {
    fn from(choice: TerraformingTool) -> Self {
        match choice {
            TerraformingTool::Raise => Self::Raise,
            TerraformingTool::Lower => Self::Lower,
            TerraformingTool::Change(terrain) => Self::Change(terrain),
        }
    }
}

impl From<TerraformingAction> for TerraformingTool {
    fn from(action: TerraformingAction) -> Self {
        match action {
            TerraformingAction::Raise => Self::Raise,
            TerraformingAction::Lower => Self::Lower,
            TerraformingAction::Change(terrain) => Self::Change(terrain),
        }
    }
}
