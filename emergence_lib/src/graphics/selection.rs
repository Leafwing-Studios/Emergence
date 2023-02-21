//! Graphics code to display the currently selected game object.

use bevy::prelude::*;

use crate::{
    asset_management::{structures::StructureHandles, terrain::TerrainHandles},
    player_interaction::selection::{CurrentSelection, HoveredTiles, ObjectInteraction},
    simulation::geometry::TilePos,
    structures::{ghost::Ghostly, StructureId},
    terrain::Terrain,
};

/// Shows which tiles are being hovered and selected.
pub(super) fn display_tile_interactions(
    current_selection: Res<CurrentSelection>,
    hovered_tiles: Res<HoveredTiles>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TerrainHandles>,
) {
    if current_selection.is_changed() || hovered_tiles.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&tile_pos);
            let selected = if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
                selected_tiles.contains_tile(tile_pos)
            } else {
                false
            };

            *material = materials.get_material(terrain, hovered, selected);
        }
    }
}

/// Replaces the material of selected structures.
// TODO: this should almost certainly use a better approach than messing with materials
pub(super) fn swap_structure_materials(
    current_selection: Res<CurrentSelection>,
    mut previously_selected_structure: Local<Option<Entity>>,
    structure_query: Query<(&StructureId, Option<&Ghostly>)>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
    mut commands: Commands,
) {
    if current_selection.is_changed() {
        // Remove the selection effect
        if let Some(previous_entity) = *previously_selected_structure {
            if let Ok((structure_id, _maybe_ghostly)) = structure_query.get(previous_entity) {
                // Remove the old scene
                commands.entity(previous_entity).despawn_descendants();

                // Trigger a re-addition of the scene
                commands
                    .entity(previous_entity)
                    .remove::<StructureId>()
                    .insert(*structure_id);

                // Clear the cache, as this has been handled
                *previously_selected_structure = None;
            }
        }

        if let CurrentSelection::Structure(current_entity) = *current_selection {
            // Cache this, so we remember to reverse it
            *previously_selected_structure = Some(current_entity);

            let selected_material_handle =
                if let Ok((_structure_id, maybe_ghostly)) = structure_query.get(current_entity) {
                    match maybe_ghostly {
                        Some(..) => structure_handles
                            .ghost_materials
                            .get(&ObjectInteraction::Selected)
                            .unwrap(),
                        None => structure_handles
                            .interaction_materials
                            .get(&ObjectInteraction::Selected)
                            .unwrap(),
                    }
                } else {
                    warn!("Selected structure could not be found!");
                    return;
                };
            for child in children.iter_descendants(current_entity) {
                if let Ok(mut existing_material_handle) = material_query.get_mut(child) {
                    *existing_material_handle = selected_material_handle.clone_weak();
                }
            }
        }
    }
}
