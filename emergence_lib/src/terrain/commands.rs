//! Methods to use [`Commands`] to manipulate terrain.

use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    scene::Scene,
};

use crate::{
    asset_management::manifest::Id,
    construction::{terraform::TerraformingAction, zoning::Zoning},
    geometry::{MapGeometry, VoxelPos},
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
};

/// An extension trait for [`Commands`] for working with terrain.
pub(crate) trait TerrainCommandsExt {
    /// Applies the given `terraforming_action` to the terrain at `voxel_pos`.
    fn apply_terraforming_action(&mut self, voxel_pos: VoxelPos, action: TerraformingAction);
}

impl<'w, 's> TerrainCommandsExt for Commands<'w, 's> {
    fn apply_terraforming_action(
        &mut self,
        voxel_pos: VoxelPos,
        terraforming_action: TerraformingAction,
    ) {
        self.add(ApplyTerraformingCommand {
            voxel_pos,
            terraforming_action,
        });
    }
}

/// A [`Command`] used to apply [`TerraformingAction`]s to a tile.
struct ApplyTerraformingCommand {
    /// The tile position at which the terrain to be despawned is found.
    voxel_pos: VoxelPos,
    /// The action to apply to the tile.
    terraforming_action: TerraformingAction,
}

impl Command for ApplyTerraformingCommand {
    fn write(self, world: &mut World) {
        // Just using system state makes satisfying the borrow checker a lot easier
        let mut system_state = SystemState::<(
            ResMut<MapGeometry>,
            Res<TerrainHandles>,
            Query<(
                &mut Id<Terrain>,
                &mut Zoning,
                &mut VoxelPos,
                &mut Handle<Scene>,
            )>,
        )>::new(world);

        let (mut map_geometry, terrain_handles, mut terrain_query) = system_state.get_mut(world);

        let terrain_entity = map_geometry.get_terrain(self.voxel_pos.hex).unwrap();

        let (mut current_terrain_id, mut zoning, mut voxel_pos, mut scene_handle) =
            terrain_query.get_mut(terrain_entity).unwrap();

        match self.terraforming_action {
            TerraformingAction::Raise => voxel_pos.height = voxel_pos.height.above(),
            TerraformingAction::Lower => {
                voxel_pos.height = voxel_pos.height.below();
            }
            TerraformingAction::Change(changed_terrain_id) => {
                *current_terrain_id = changed_terrain_id;
            }
        };

        // We can't do this above, as we need to drop the previous query before borrowing from the world again
        if let TerraformingAction::Change(changed_terrain_id) = self.terraforming_action {
            *scene_handle = terrain_handles
                .scenes
                .get(&changed_terrain_id)
                .unwrap()
                .clone_weak();
        }

        map_geometry.update_height(voxel_pos.hex, voxel_pos.height);
        *zoning = Zoning::None;
    }
}
