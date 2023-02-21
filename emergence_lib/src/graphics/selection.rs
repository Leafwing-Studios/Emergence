//! Graphics code to display the currently selected game object.

use bevy::prelude::*;

use crate::{
    asset_management::terrain::TerrainHandles,
    player_interaction::selection::{CurrentSelection, HoveredTiles},
    simulation::geometry::TilePos,
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
