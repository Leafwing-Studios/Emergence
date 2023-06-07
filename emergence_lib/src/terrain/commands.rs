//! Methods to use [`Commands`] to manipulate terrain.

use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    scene::Scene,
};

use crate::{
    asset_management::manifest::Id,
    construction::{
        ghosts::{GhostHandles, GhostKind},
        terraform::{GhostTerrainBundle, TerraformingAction, TerrainPreviewBundle},
        zoning::Zoning,
    },
    geometry::{Height, MapGeometry, VoxelPos},
    graphics::InheritedMaterial,
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
};

use super::{terrain_manifest::TerrainManifest, TerrainBundle};

/// An extension trait for [`Commands`] for working with terrain.
pub(crate) trait TerrainCommandsExt {
    /// Adds the appropriate terrain bundle and children to an entity with a [`TerrainPrototype`].
    fn hydrate_terrain(&mut self, entity: Entity, height: Height, terrain_id: Id<Terrain>);

    /// Spawns a ghost that previews the action given by `terraforming_action` at `voxel_pos`.
    ///
    /// Replaces any existing ghost.
    fn spawn_ghost_terrain(
        &mut self,
        voxel_pos: VoxelPos,
        terrain_id: Id<Terrain>,
        terraforming_action: TerraformingAction,
    );

    /// Despawns any ghost at the provided `voxel_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_ghost_terrain(&mut self, voxel_pos: VoxelPos);

    /// Spawns a preview that previews the action given by `terraforming_action` at `voxel_pos`.
    fn spawn_preview_terrain(
        &mut self,
        voxel_pos: VoxelPos,
        terrain_id: Id<Terrain>,
        terraforming_action: TerraformingAction,
    );

    /// Applies the given `terraforming_action` to the terrain at `voxel_pos`.
    fn apply_terraforming_action(&mut self, voxel_pos: VoxelPos, action: TerraformingAction);
}

impl<'w, 's> TerrainCommandsExt for Commands<'w, 's> {
    fn hydrate_terrain(&mut self, entity: Entity, height: Height, terrain_id: Id<Terrain>) {
        self.add(HydrateTerrainCommand {
            entity,
            height,
            terrain_id,
        });
    }

    fn spawn_ghost_terrain(
        &mut self,
        voxel_pos: VoxelPos,
        terrain_id: Id<Terrain>,
        terraforming_action: TerraformingAction,
    ) {
        self.add(SpawnTerrainGhostCommand {
            voxel_pos,
            terrain_id,
            terraforming_action,
            ghost_kind: GhostKind::Ghost,
        });
    }

    fn despawn_ghost_terrain(&mut self, voxel_pos: VoxelPos) {
        self.add(DespawnGhostCommand { voxel_pos });
    }

    fn spawn_preview_terrain(
        &mut self,
        voxel_pos: VoxelPos,
        terrain_id: Id<Terrain>,
        terraforming_action: TerraformingAction,
    ) {
        self.add(SpawnTerrainGhostCommand {
            voxel_pos,
            terrain_id,
            terraforming_action,
            ghost_kind: GhostKind::Preview,
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
    pub(crate) height: Height,
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
        let new_voxel_pos = VoxelPos::new(existing_voxel_pos.hex, self.height);

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

/// A [`Command`] used to spawn a ghost via [`TerrainCommandsExt`].
struct SpawnTerrainGhostCommand {
    /// The tile position at which the ghost should be spawned.
    voxel_pos: VoxelPos,
    /// The terrain type that the ghost represents.
    terrain_id: Id<Terrain>,
    /// The action that the ghost represents.
    terraforming_action: TerraformingAction,
    /// What kind of ghost this is.
    ghost_kind: GhostKind,
}

impl Command for SpawnTerrainGhostCommand {
    fn write(self, world: &mut World) {
        let map_geometry = world.resource::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !map_geometry.is_valid(self.voxel_pos.hex) {
            return;
        }

        // Remove any existing ghost terrain
        if let Some(ghost_entity) = map_geometry.get_ghost_terrain(self.voxel_pos) {
            if world.entities().contains(ghost_entity) && self.ghost_kind == GhostKind::Ghost {
                world.entity_mut(ghost_entity).despawn_recursive();
                let mut map_geometry = world.resource_mut::<MapGeometry>();
                map_geometry.remove_ghost_terrain(self.voxel_pos);
            }
        }

        let map_geometry = world.resource::<MapGeometry>();
        let scene_handle = world
            .resource::<TerrainHandles>()
            .scenes
            .get(&self.terrain_id)
            .unwrap()
            .clone_weak();

        let ghost_handles = world.resource::<GhostHandles>();
        let ghost_material = ghost_handles.get_material(self.ghost_kind);

        let inherited_material = InheritedMaterial(ghost_material);
        let current_height = map_geometry.get_height(self.voxel_pos.hex).unwrap();
        let new_height = match self.terraforming_action {
            TerraformingAction::Raise => current_height + Height(1.),
            TerraformingAction::Lower => current_height - Height(1.),
            _ => current_height,
        };

        let mut world_pos = self.voxel_pos.into_world_pos(map_geometry);
        world_pos.y = new_height.into_world_pos();

        match self.ghost_kind {
            GhostKind::Ghost => {
                let input_inventory = self.terraforming_action.input_inventory();
                let output_inventory = self.terraforming_action.output_inventory();

                let ghost_entity = world
                    .spawn(GhostTerrainBundle::new(
                        self.terraforming_action,
                        self.voxel_pos,
                        scene_handle,
                        inherited_material,
                        world_pos,
                        input_inventory,
                        output_inventory,
                    ))
                    .id();

                // Update the index to reflect the new state
                let mut map_geometry = world.resource_mut::<MapGeometry>();
                map_geometry.add_ghost_terrain(ghost_entity, self.voxel_pos);
            }
            GhostKind::Preview => {
                // Previews are not indexed, and are instead just spawned and despawned as needed
                world.spawn(TerrainPreviewBundle::new(
                    self.voxel_pos,
                    self.terraforming_action,
                    scene_handle,
                    inherited_material,
                    world_pos,
                ));
            }
            _ => unreachable!("Invalid ghost kind provided."),
        }
    }
}

/// A [`Command`] used to despawn a ghost via [`TerrainCommandsExt`].
struct DespawnGhostCommand {
    /// The tile position at which the terrain to be despawned is found.
    voxel_pos: VoxelPos,
}

impl Command for DespawnGhostCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();
        let maybe_entity = geometry.remove_ghost_terrain(self.voxel_pos);

        // Check that there's something there to despawn
        let Some(ghost_entity) = maybe_entity else {
            return;
        };

        // Make sure to despawn all children, which represent the meshes stored in the loaded gltf scene.
        world.entity_mut(ghost_entity).despawn_recursive();
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
            TerraformingAction::Raise => {
                voxel_pos.height = (voxel_pos.height + 1).min(Height::MAX.0 as i32)
            }
            TerraformingAction::Lower => {
                voxel_pos.height = (voxel_pos.height - 1).max(Height::MIN.0 as i32)
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

        map_geometry.update_height(voxel_pos.hex, voxel_pos.height());
        *zoning = Zoning::None;
    }
}
