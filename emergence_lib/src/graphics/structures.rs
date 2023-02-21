//! Graphics and animation code for structures.

use bevy::prelude::*;

use crate::{
    asset_management::structures::StructureHandles,
    simulation::geometry::{MapGeometry, TilePos},
    structures::StructureId,
};

/// Adds rendering components to every spawned structure, real or otherwise
pub(super) fn populate_structures(
    new_structures: Query<(Entity, &TilePos, &StructureId), Added<StructureId>>,
    mut commands: Commands,
    structure_handles: Res<StructureHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (entity, tile_pos, structure_id) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        let scene_handle = structure_handles.scenes.get(structure_id).unwrap();

        commands
            .entity(entity)
            .insert(SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_xyz(pos.x, *terrain_height, pos.y),
                ..default()
            })
            .insert(structure_handles.picking_mesh.clone_weak());
    }
}
