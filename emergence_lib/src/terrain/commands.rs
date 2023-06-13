//! Methods to use [`Commands`] to manipulate terrain.

use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    scene::Scene,
};

use crate::{
    asset_management::manifest::Id,
    construction::{terraform::TerraformingAction, zoning::Zoning},
    geometry::{DiscreteHeight, MapGeometry, VoxelPos},
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
};

use super::{terrain_manifest::TerrainManifest, TerrainBundle};

/// An extension trait for [`Commands`] for working with terrain.
pub(crate) trait TerrainCommandsExt {
    /// Adds the appropriate terrain bundle and children to an entity with a [`TerrainPrototype`].
    fn hydrate_terrain(&mut self, entity: Entity, height: DiscreteHeight, terrain_id: Id<Terrain>);

    /// Applies the given `terraforming_action` to the terrain at `voxel_pos`.
    fn apply_terraforming_action(&mut self, voxel_pos: VoxelPos, action: TerraformingAction);
}

impl<'w, 's> TerrainCommandsExt for Commands<'w, 's> {
    fn hydrate_terrain(&mut self, entity: Entity, height: DiscreteHeight, terrain_id: Id<Terrain>) {
        self.add(HydrateTerrainCommand {
            entity,
            height,
            terrain_id,
        });
    }

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

/// Hydrates a new terrain tile initialized by the [`MapGeometry`].
///
/// The order of the chidlren *must* be:
/// 0: column
/// 1: scene root
pub(crate) struct HydrateTerrainCommand {
    /// The entity to modify
    pub(crate) entity: Entity,
    /// The new height of the tile
    pub(crate) height: DiscreteHeight,
    /// The type of terrain
    pub(crate) terrain_id: Id<Terrain>,
}

impl Command for HydrateTerrainCommand {
    fn write(self, world: &mut World) {
        let handles = world.resource::<TerrainHandles>();
        let scene_handle = handles.scenes.get(&self.terrain_id).unwrap().clone_weak();
        let mesh = handles.topper_mesh.clone_weak();

        // Drop the borrow so the borrow checker is happy
        let map_geometry = world.resource::<MapGeometry>();
        let terrain_manifest = world.resource::<TerrainManifest>();

        let existing_voxel_pos: VoxelPos = *world.get(self.entity).unwrap();
        let new_voxel_pos = VoxelPos {
            hex: existing_voxel_pos.hex,
            height: self.height,
        };

        // Insert the TerrainBundle
        let terrain_bundle = TerrainBundle::new(
            self.terrain_id,
            new_voxel_pos,
            scene_handle,
            mesh,
            terrain_manifest,
            map_geometry,
        );

        // This overwrites the existing VoxelPos component
        world.entity_mut(self.entity).insert(terrain_bundle);

        // Spawn the column as the 0th child of the tile entity
        // The scene bundle will be added as the first child
        let handles = world.resource::<TerrainHandles>();
        let column_bundle = PbrBundle {
            mesh: handles.column_mesh.clone_weak(),
            material: handles.column_material.clone_weak(),
            ..Default::default()
        };

        let hex_column = world.spawn(column_bundle).id();
        world.entity_mut(self.entity).add_child(hex_column);

        // Update the index of what terrain is where
        let mut map_geometry = world.resource_mut::<MapGeometry>();
        map_geometry.update_height(existing_voxel_pos.hex, self.height);
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
