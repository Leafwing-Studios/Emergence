//! Zoning is used to indicate that a tile should contain the specified structure.

use bevy::prelude::*;
use core::fmt::Display;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, construction::MarkedForDemolition},
    terrain::Terrain,
};

use super::{
    clipboard::{Clipboard, ClipboardData},
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
                .in_set(InteractionSystem::ApplyZoning)
                .after(InteractionSystem::SelectTiles)
                .after(InteractionSystem::SetClipboard),
        )
        .add_system(
            generate_ghosts_from_zoning
                .in_set(InteractionSystem::ManagePreviews)
                .after(InteractionSystem::ApplyZoning),
        )
        // Must run after crafting emitters in order to wipe out their signals
        .add_system(keep_tiles_clear.after(crate::structures::crafting::set_emitter));
    }
}

/// The zoning of a given tile, which specifies which structure *should* be built there.
#[derive(Component, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Zoning {
    /// The provided structure should be built on this tile.
    Structure(ClipboardData),
    /// No zoning is set.
    None,
    /// Zoning is set to keep the tile clear.
    KeepClear,
}

impl Display for Zoning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Zoning::Structure(clipboard_data) => {
                let id = clipboard_data.structure_id;
                format!("{id}")
            }
            Zoning::None => "None".to_string(),
            Zoning::KeepClear => "Keep Clear".to_string(),
        };

        write!(f, "{str}")
    }
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
    map_geometry: Res<MapGeometry>,
) {
    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        let selected_tiles = match &*current_selection {
            CurrentSelection::Terrain(selected_tiles) => selected_tiles.clone(),
            _ => {
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

        // Try to remove everything at the location
        if actions.pressed(PlayerAction::KeepClear) {
            for terrain_entity in relevant_terrain_entities {
                let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                *zoning = Zoning::KeepClear;
            }

            // Don't try to clear and zone in the same frame
            return;
        }

        // Explicitly clear the selection
        if actions.pressed(PlayerAction::ClearZoning) {
            for terrain_entity in relevant_terrain_entities {
                let mut zoning = terrain_query.get_mut(terrain_entity).unwrap();
                *zoning = Zoning::None;
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
                    *zoning = Zoning::None;
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

/// Spawn and despawn ghosts based on zoning.
fn generate_ghosts_from_zoning(
    // We cannot use change detection here, or tiles would not be kept clear when built upon after zoning is set
    mut terrain_query: Query<(&mut Zoning, &TilePos, &Terrain)>,
    structure_manifest: Res<StructureManifest>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (mut zoning, &tile_pos, terrain) in terrain_query.iter_mut() {
        // Reborrowing here would trigger change detection, causing this system to constantly check
        match zoning.bypass_change_detection() {
            Zoning::Structure(clipboard_data) => {
                let structure_data = structure_manifest.get(clipboard_data.structure_id);
                if structure_data.allowed_terrain_types().contains(terrain) {
                    commands.spawn_ghost(tile_pos, clipboard_data.clone())
                } else {
                    *zoning = Zoning::None;
                    // We bypassed change detection above, so need to manually trigger it here.
                    zoning.set_changed();
                }
            }
            Zoning::None => commands.despawn_ghost(tile_pos),
            Zoning::KeepClear => {
                if let Some(structure_entity) = map_geometry.structure_index.get(&tile_pos) {
                    commands
                        .entity(*structure_entity)
                        .insert(MarkedForDemolition);
                }
            }
        };
    }
}

/// Keeps marked tiles clear by sending removal signals from structures that are marked for removal
fn keep_tiles_clear(
    mut structure_query: Query<(&mut Emitter, &Id<Structure>), With<MarkedForDemolition>>,
) {
    for (mut doomed_emitter, &structure_id) in structure_query.iter_mut() {
        doomed_emitter.signals = vec![(
            SignalType::Demolish(structure_id),
            SignalStrength::new(100.),
        )];
    }
}
