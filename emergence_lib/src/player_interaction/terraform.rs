//! Tools to alter the terrain type and height.

use bevy::prelude::*;

use crate::{
    asset_management::{
        manifest::{Id, Terrain, TerrainManifest},
        terrain::TerrainHandles,
    },
    simulation::geometry::{Height, TilePos},
    structures::commands::StructureCommandsExt,
};

use super::InteractionSystem;

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((apply_terraforming,).in_set(InteractionSystem::ApplyTerraforming));
    }
}

/// An option for how to terraform the world.
///
/// When `Zoning` is set, this is added  as a component added to terrain entities that marks them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingChoice {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided Id<Terrain>.
    Change(Id<Terrain>),
}

impl TerraformingChoice {
    /// Pretty formatting for this type
    pub(crate) fn display(&self, terrain_manifest: &TerrainManifest) -> String {
        match self {
            TerraformingChoice::Raise => "Raise".to_string(),
            TerraformingChoice::Lower => "Lower".to_string(),
            TerraformingChoice::Change(terrain_id) => {
                terrain_manifest.name(*terrain_id).to_string()
            }
        }
    }
}

/// Changes the terrain to match the [`Terraform`] component
fn apply_terraforming(
    mut query: Query<(
        Entity,
        &TerraformingChoice,
        &TilePos,
        &mut Id<Terrain>,
        &mut Height,
        &mut Handle<Scene>,
    )>,
    terrain_handles: Res<TerrainHandles>,
    mut commands: Commands,
) {
    // TODO: this should take work.
    for (entity, terraform, tile_pos, mut terrain, mut height, mut scene_handle) in query.iter_mut()
    {
        match terraform {
            TerraformingChoice::Raise => *height += Height(1),
            TerraformingChoice::Lower => *height -= Height(1),
            TerraformingChoice::Change(target_terrain_type) => {
                *terrain = *target_terrain_type;
                *scene_handle = terrain_handles
                    .scenes
                    .get(target_terrain_type)
                    .unwrap()
                    .clone_weak();
            }
        }

        // Don't keep the component around
        commands.entity(entity).remove::<TerraformingChoice>();

        // Despawn any structures here; terraforming can't be done with roots growing into stuff!
        commands.despawn_structure(*tile_pos);
    }
}
