use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    simulation::geometry::{MapGeometry, TilePos},
    structures::commands::StructureCommandsExt,
    terrain::Terrain,
};

use super::{
    clipboard::{Clipboard, ClipboardItem},
    cursor::CursorPos,
    tile_selection::SelectedTiles,
    InteractionSystem, SelectionAction,
};

/// Code and data for setting zoning of areas for construction.
pub(super) struct ZoningPlugin;

impl Plugin for ZoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            set_zoning
                .label(InteractionSystem::ApplyZoning)
                .after(InteractionSystem::SelectTiles)
                .after(InteractionSystem::SetClipboard),
        )
        .add_system(
            act_on_zoning
                .label(InteractionSystem::ManageGhosts)
                .after(InteractionSystem::ApplyZoning),
        );
    }
}

/// The zoning of a given tile, which specifies which structure *should* be built there.
#[derive(Component, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Zoning {
    /// The provided structure should be built on this tile.
    Structure(ClipboardItem),
    /// No zoning is set.
    None,
    /// Zoning is set to keep the tile clear.
    Clear,
}

/// Applies zoning to an area, causing structures to be created (or removed) there.
///
/// This system also handles the "paste" functionality.
fn set_zoning(
    cursor: Res<CursorPos>,
    actions: Res<ActionState<SelectionAction>>,
    clipboard: Res<Clipboard>,
    mut terrain_query: Query<&mut Zoning, With<Terrain>>,
    selected_tiles: Res<SelectedTiles>,
    map_geometry: Res<MapGeometry>,
) {
    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        let relevant_terrain_entities: Vec<Entity> = if selected_tiles.is_empty() {
            vec![*map_geometry.terrain_index.get(&cursor_tile_pos).unwrap()]
        } else {
            selected_tiles
                .selection()
                .iter()
                .map(|tile_pos| *map_geometry.terrain_index.get(tile_pos).unwrap())
                .collect()
        };

        // Explicitly clear the selection
        if actions.pressed(SelectionAction::ClearZoning) {
            for terrain_entity in relevant_terrain_entities {
                let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                *zoning = Zoning::Clear;
            }

            // Don't try to clear and zone in the same frame
            return;
        }

        // Apply zoning
        if actions.pressed(SelectionAction::Zone) {
            if clipboard.is_empty() {
                // Clear zoning
                for terrain_entity in relevant_terrain_entities {
                    let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                    *zoning = Zoning::Clear;
                }
            // Zone using the single selected structure
            } else if clipboard.len() == 1 {
                let clipboard_item = clipboard.values().next().unwrap();
                for terrain_entity in relevant_terrain_entities {
                    let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                    *zoning = Zoning::Structure(clipboard_item.clone());
                }
            // Paste the selection
            } else {
                for (tile_pos, clipboard_item) in clipboard.offset_positions(cursor_tile_pos) {
                    // Avoid trying to operate on terrain that doesn't exist
                    if let Some(&terrain_entity) = map_geometry.terrain_index.get(&tile_pos) {
                        let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                        *zoning = Zoning::Structure(clipboard_item.clone());
                    }
                }
            }
        }
    }
}

/// Spawn and despawn structures based on their zoning.
fn act_on_zoning(
    terrain_query: Query<(&Zoning, &TilePos), (With<Terrain>, Changed<Zoning>)>,
    mut commands: Commands,
) {
    for (zoning, &tile_pos) in terrain_query.iter() {
        match zoning {
            Zoning::Structure(item) => commands.spawn_structure(tile_pos, item.clone()),
            Zoning::None => (), // Do nothing
            Zoning::Clear => commands.despawn_structure(tile_pos),
        };
    }
}
