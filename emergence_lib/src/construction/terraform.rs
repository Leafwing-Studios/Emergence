//! Tools to alter the terrain type and height.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    player_interaction::InteractionSystem,
    simulation::{
        geometry::{MapGeometry, TilePos},
        SimulationSet,
    },
    terrain::{
        commands::TerrainCommandsExt,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

use super::{demolition::MarkedForDemolition, zoning::Zoning, ConstructionData};

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (spawn_terraforming_ghosts,)
                .in_set(InteractionSystem::ApplyTerraforming)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

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

/// Changes the terrain to match the [`MarkedForTerraforming`] component
fn spawn_terraforming_ghosts(
    mut terrain_query: Query<(&TilePos, Ref<Zoning>, &Id<Terrain>)>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    for (&tile_pos, zoning, current_terrain_id) in terrain_query.iter_mut() {
        if zoning.is_changed() {
            if let Zoning::Terraform(terraforming_action) = *zoning {
                // We neeed to use the model for the terrain we're changing to, not the current one
                let terrain_id = match terraforming_action {
                    TerraformingAction::Change(terrain_id) => terrain_id,
                    _ => *current_terrain_id,
                };

                commands.spawn_ghost_terrain(tile_pos, terrain_id, terraforming_action);

                // Mark any structures that are here as needing to be demolished
                // Terraforming can't be done with roots growing into stuff!
                if let Some(structure_entity) = map_geometry.get_structure(tile_pos) {
                    commands
                        .entity(structure_entity)
                        .insert(MarkedForDemolition);
                }
            }
        }
    }
}
