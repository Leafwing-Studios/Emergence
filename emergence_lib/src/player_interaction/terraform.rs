//! Tools to alter the terrain type and height.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, TilePos},
    structures::commands::StructureCommandsExt,
    terrain::{
        terrain_assets::TerrainHandles,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

use super::{zoning::Zoning, InteractionSystem};

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((apply_terraforming,).in_set(InteractionSystem::ApplyTerraforming));
    }
}

/// An option for how to terraform the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingChoice {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided [`Id<Terrain>`].
    Change(Id<Terrain>),
}

impl TerraformingChoice {
    /// Converts `self` into a [`MarkedForTerraforming`] component.
    pub(crate) fn into_mark(
        self,
        current_height: Height,
        current_material: Id<Terrain>,
    ) -> MarkedForTerraforming {
        match self {
            TerraformingChoice::Raise => MarkedForTerraforming {
                target_height: current_height + Height(1),
                target_material: current_material,
            },
            TerraformingChoice::Lower => MarkedForTerraforming {
                target_height: current_height - Height(1),
                target_material: current_material,
            },
            TerraformingChoice::Change(target_material) => MarkedForTerraforming {
                target_height: current_height,
                target_material,
            },
        }
    }
}

/// When `Zoning` is set, this is added  as a component added to terrain entities that marks them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MarkedForTerraforming {
    /// The desired height of this tile
    target_height: Height,
    /// The desired terrain material of this tile
    target_material: Id<Terrain>,
}

impl MarkedForTerraforming {
    /// Pretty formatting for this type
    pub(crate) fn display(&self, terrain_manifest: &TerrainManifest) -> String {
        format!(
            "Terraform: Height {}, Terrain Type {}",
            self.target_height,
            terrain_manifest.name(self.target_material)
        )
    }
}

/// Changes the terrain to match the [`MarkedForTerraforming`] component
fn apply_terraforming(
    mut query: Query<(
        Entity,
        &MarkedForTerraforming,
        &TilePos,
        &mut Zoning,
        &mut Id<Terrain>,
        &mut Height,
        &mut Handle<Scene>,
    )>,
    terrain_handles: Res<TerrainHandles>,
    mut commands: Commands,
) {
    for (
        entity,
        marked_for_terraforming,
        tile_pos,
        mut zoning,
        mut terrain,
        mut height,
        mut scene_handle,
    ) in query.iter_mut()
    {
        // TODO: this should take work.
        *height = marked_for_terraforming.target_height;
        *terrain = marked_for_terraforming.target_material;
        *scene_handle = terrain_handles
            .scenes
            .get(&marked_for_terraforming.target_material)
            .unwrap()
            .clone_weak();

        if *height == marked_for_terraforming.target_height
            && *terrain == marked_for_terraforming.target_material
        {
            // Don't keep the components around once we've completed our action
            commands.entity(entity).remove::<MarkedForTerraforming>();
            // Reset the zoning when we're done
            *zoning = Zoning::None;
        }

        // Despawn any structures here; terraforming can't be done with roots growing into stuff!
        commands.despawn_structure(*tile_pos);
    }
}
