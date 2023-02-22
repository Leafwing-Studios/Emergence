//! Graphics code for terrain.

use bevy::prelude::*;

use crate::{
    asset_management::terrain::TerrainHandles,
    simulation::geometry::{MapGeometry, TilePos},
    terrain::Terrain,
};

/// Adds rendering components to every spawned terrain tile
pub(super) fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    handles: Res<TerrainHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, tile_pos, terrain) in new_terrain.iter() {
        let world_pos = tile_pos.into_world_pos(&map_geometry);

        commands.entity(terrain_entity).insert(PbrBundle {
            mesh: handles.mesh.clone_weak(),
            material: handles.terrain_materials.get(terrain).unwrap().clone_weak(),
            transform: Transform::from_xyz(world_pos.x, 0.0, world_pos.z).with_scale(Vec3 {
                x: 1.,
                y: world_pos.y,
                z: 1.,
            }),
            ..default()
        });
    }
}
