use bevy::prelude::*;

use crate::{
    asset_management::structures::StructureHandles,
    simulation::geometry::{MapGeometry, TilePos},
    structures::{
        ghost::{Ghost, Preview},
        StructureId,
    },
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

/// Modifies the material of any entities spawned due to a ghost structure.
pub(super) fn change_ghost_material(
    ghost_query: Query<Entity, With<Ghost>>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
) {
    for ghost_entity in ghost_query.iter() {
        for child in children.iter_descendants(ghost_entity) {
            if let Ok(mut material) = material_query.get_mut(child) {
                *material = structure_handles.ghost_material.clone_weak();
            }
        }
    }
}

/// Modifies the material of any entities spawned due to a ghost structure.
pub(super) fn change_preview_material(
    ghost_query: Query<Entity, With<Preview>>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
) {
    for ghost_entity in ghost_query.iter() {
        for child in children.iter_descendants(ghost_entity) {
            if let Ok(mut material) = material_query.get_mut(child) {
                *material = structure_handles.preview_material.clone_weak();
            }
        }
    }
}
