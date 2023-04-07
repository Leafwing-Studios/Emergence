//! Tools to alter the terrain type and height.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

use super::ConstructionData;

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
    /// The construction requirements for this action.
    // TODO: actually require materials
    pub(crate) fn construction_data(&self) -> ConstructionData {
        match self {
            Self::Raise => ConstructionData {
                work: Some(Duration::from_secs(5)),
                ..Default::default()
            },
            Self::Lower => ConstructionData {
                work: Some(Duration::from_secs(5)),
                ..Default::default()
            },
            Self::Change(_) => ConstructionData {
                work: Some(Duration::from_secs(5)),
                ..Default::default()
            },
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
