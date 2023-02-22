//! Graphics and animation code for structures.

use bevy::prelude::*;

use crate::{
    asset_management::structures::StructureHandles,
    player_interaction::selection::ObjectInteraction,
    simulation::geometry::{MapGeometry, TilePos},
    structures::{ghost::Ghostly, StructureId},
};

/// Adds rendering components to every spawned structure, real or otherwise
pub(super) fn populate_structures(
    new_structures: Query<(Entity, &TilePos, &StructureId), Added<StructureId>>,
    mut commands: Commands,
    structure_handles: Res<StructureHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (entity, tile_pos, structure_id) in new_structures.iter() {
        let scene_handle = structure_handles.scenes.get(structure_id).unwrap();

        commands
            .entity(entity)
            .insert(SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(tile_pos.into_world_pos(&*map_geometry)),
                ..default()
            })
            .insert(structure_handles.picking_mesh.clone_weak());
    }
}

/// Modifies the material of any structures based on their interaction state.
pub(super) fn change_structure_material(
    root_structure_query: Query<(Entity, &ObjectInteraction, Option<&Ghostly>), With<StructureId>>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
) {
    for (root_entity, object_interaction, maybe_ghostly) in root_structure_query.iter() {
        for child in children.iter_descendants(root_entity) {
            if let Ok(mut material) = material_query.get_mut(child) {
                let maybe_material_handle = match maybe_ghostly {
                    Some(..) => structure_handles.ghost_materials.get(object_interaction),
                    None => structure_handles
                        .interaction_materials
                        .get(object_interaction),
                };

                // FIXME: how do we restore the materials back to their original form??
                if let Some(new_handle) = maybe_material_handle {
                    *material = new_handle.clone_weak();
                }
            }
        }
    }
}
