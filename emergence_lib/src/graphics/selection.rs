//! Graphics code to display the currently selected game object.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{Id, Terrain},
    player_interaction::selection::{CurrentSelection, HoveredTiles},
    simulation::geometry::TilePos,
};

/// Shows which tiles are being hovered and selected.
pub(super) fn display_tile_interactions(
    current_selection: Res<CurrentSelection>,
    hovered_tiles: Res<HoveredTiles>,
    mut terrain_query: Query<(&Id<Terrain>, &TilePos)>,
) {
    if current_selection.is_changed() || hovered_tiles.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&tile_pos);
            let selected = if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
                selected_tiles.contains_tile(tile_pos)
            } else {
                false
            };

            // FIXME: unbreak tile selection display
            //*material = materials.get_material(terrain, hovered, selected);
        }
    }
}
