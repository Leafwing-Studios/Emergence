//! Zoning is used to indicate that a tile should contain the specified structure.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    simulation::geometry::{Facing, MapGeometry, TilePos},
    structures::{
        commands::StructureCommandsExt, crafting::InputInventory, ghost::Ghost, StructureId,
    },
    terrain::Terrain,
};

use super::{
    clipboard::{Clipboard, StructureData},
    cursor::CursorPos,
    selection::{CurrentSelection, SelectedTiles},
    InteractionSystem, PlayerAction,
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
                .label(InteractionSystem::ManagePreviews)
                .after(InteractionSystem::ApplyZoning),
        )
        .add_system(turn_ghosts_into_structures);
    }
}

/// The zoning of a given tile, which specifies which structure *should* be built there.
#[derive(Component, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Zoning {
    /// The provided structure should be built on this tile.
    Structure(StructureData),
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
    actions: Res<ActionState<PlayerAction>>,
    clipboard: Res<Clipboard>,
    mut terrain_query: Query<&mut Zoning, With<Terrain>>,
    current_selection: Res<CurrentSelection>,
    tile_pos_query: Query<&TilePos>,
    map_geometry: Res<MapGeometry>,
) {
    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        let selected_tiles = match &*current_selection {
            CurrentSelection::Ghost(entity) | CurrentSelection::Structure(entity) => {
                let selection_tile_pos = *tile_pos_query.get(*entity).unwrap();
                let mut selected_tiles = SelectedTiles::default();
                selected_tiles.add_tile(selection_tile_pos);
                selected_tiles
            }
            CurrentSelection::Terrain(selected_tiles) => selected_tiles.clone(),
            CurrentSelection::None | CurrentSelection::Unit(_) => {
                let mut selected_tiles = SelectedTiles::default();
                selected_tiles.add_tile(cursor_tile_pos);
                selected_tiles
            }
        };

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
        if actions.pressed(PlayerAction::ClearZoning) {
            for terrain_entity in relevant_terrain_entities {
                let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                *zoning = Zoning::Clear;
            }

            // Don't try to clear and zone in the same frame
            return;
        }

        // Apply zoning
        if actions.pressed(PlayerAction::Zone) {
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
            Zoning::Structure(item) => commands.spawn_ghost(tile_pos, item.clone()),
            Zoning::None => commands.despawn_ghost(tile_pos),
            // TODO: this should also take delayed effect
            Zoning::Clear => commands.despawn_structure(tile_pos),
        };
    }
}

/// Transforms ghosts into structures once all of their construction materials have been supplied.
fn turn_ghosts_into_structures(
    ghost_query: Query<(&InputInventory, &TilePos, &StructureId, &Facing), With<Ghost>>,
    mut commands: Commands,
) {
    for (input_inventory, &tile_pos, &structure_id, &facing) in ghost_query.iter() {
        if input_inventory.is_full() {
            commands.despawn_ghost(tile_pos);
            commands.spawn_structure(
                tile_pos,
                StructureData {
                    structure_id,
                    facing,
                },
            );
        }
    }
}
