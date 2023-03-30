//! Zoning is used to indicate that a tile should contain the specified structure.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest, Terrain, TerrainManifest},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{Height, MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, construction::MarkedForDemolition},
};

use super::{
    clipboard::{Clipboard, ClipboardData},
    cursor::CursorPos,
    selection::CurrentSelection,
    terraform::MarkedForTerraforming,
    InteractionSystem, PlayerAction,
};

/// Code and data for setting zoning of areas for construction.
pub(super) struct ZoningPlugin;

impl Plugin for ZoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (mark_for_demolition, set_zoning)
                .in_set(InteractionSystem::ApplyZoning)
                .after(InteractionSystem::SelectTiles)
                .after(InteractionSystem::SetClipboard),
        )
        .add_system(
            mark_based_on_zoning
                .in_set(InteractionSystem::ManagePreviews)
                .after(InteractionSystem::ApplyZoning),
        )
        // Must run after crafting emitters in order to wipe out their signals
        .add_system(keep_tiles_clear.after(crate::structures::crafting::set_crafting_emitter));
    }
}

/// The zoning of a given tile, which specifies which structure *should* be built there.
#[derive(Component, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Zoning {
    /// The provided structure should be built on this tile.
    Structure(ClipboardData),
    /// The provided terraforming should be applied to this tile.
    Terraform(MarkedForTerraforming),
    /// No zoning is set.
    None,
    /// Zoning is set to keep the tile clear.
    KeepClear,
}

impl Zoning {
    /// Pretty formatting for this type.
    pub(crate) fn display(
        &self,
        structure_manifest: &StructureManifest,
        terrain_manifest: &TerrainManifest,
    ) -> String {
        match self {
            Zoning::Structure(clipboard_data) => structure_manifest
                .name(clipboard_data.structure_id)
                .to_string(),
            Zoning::Terraform(mark) => mark.display(terrain_manifest),
            Zoning::None => "None".to_string(),
            Zoning::KeepClear => "Keep Clear".to_string(),
        }
    }
}

/// Applies zoning to an area, causing structures to be created (or removed) there.
///
/// This system also handles the "paste" functionality.
fn set_zoning(
    cursor_pos: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    clipboard: Res<Clipboard>,
    mut terrain_query: Query<(&mut Zoning, &Height, &Id<Terrain>)>,
    current_selection: Res<CurrentSelection>,
    map_geometry: Res<MapGeometry>,
) {
    let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
    let relevant_terrain_entities = relevant_tiles.entities(&map_geometry);

    // Try to remove everything at the location
    if actions.pressed(PlayerAction::KeepClear) {
        for terrain_entity in relevant_terrain_entities {
            let (mut zoning, ..) = terrain_query.get_mut(terrain_entity).unwrap();
            *zoning = Zoning::KeepClear;
        }

        // Don't try to clear and zone in the same frame
        return;
    }

    // Explicitly clear the selection
    if actions.pressed(PlayerAction::ClearZoning) {
        for terrain_entity in relevant_terrain_entities {
            let (mut zoning, ..) = terrain_query.get_mut(terrain_entity).unwrap();
            *zoning = Zoning::None;
        }

        // Don't try to clear and zone in the same frame
        return;
    }

    // Apply zoning
    if actions.pressed(PlayerAction::Paste)
        || actions.pressed(PlayerAction::Select) && !clipboard.is_empty()
    {
        match &*clipboard {
            Clipboard::Terraform(terraform_choice) => {
                for terrain_entity in relevant_terrain_entities {
                    let (mut zoning, &current_height, &current_material) =
                        terrain_query.get_mut(terrain_entity).unwrap();
                    *zoning = Zoning::Terraform(
                        terraform_choice.into_mark(current_height, current_material),
                    );
                }
            }
            Clipboard::Structures(map) => {
                if map.is_empty() {
                    // Clear zoning
                    // Zone using the single selected structure
                } else if map.len() == 1 {
                    let clipboard_item = map.values().next().unwrap();
                    for terrain_entity in relevant_terrain_entities {
                        let (mut zoning, ..) = terrain_query.get_mut(terrain_entity).unwrap();
                        *zoning = Zoning::Structure(clipboard_item.clone());
                    }
                // Paste the selection
                } else {
                    let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() else {
                    return;
                };

                    for (tile_pos, clipboard_item) in clipboard.offset_positions(cursor_tile_pos) {
                        // Avoid trying to operate on terrain that doesn't exist
                        if let Some(terrain_entity) = map_geometry.get_terrain(tile_pos) {
                            let (mut zoning, ..) = terrain_query.get_mut(terrain_entity).unwrap();
                            *zoning = Zoning::Structure(clipboard_item.clone());
                        }
                    }
                }
            }
            Clipboard::Empty => {
                for terrain_entity in relevant_terrain_entities {
                    let (mut zoning, ..) = terrain_query.get_mut(terrain_entity).unwrap();
                    *zoning = Zoning::None;
                }
            }
        }
    }
}

/// Mark the selected structure for deletion.
///
/// Note that this is distinct from setting the tile to [`Zoning::KeepClear`], as it does not persist.
fn mark_for_demolition(
    player_actions: Res<ActionState<PlayerAction>>,
    current_selection: Res<CurrentSelection>,
    mut commands: Commands,
) {
    if player_actions.just_pressed(PlayerAction::KeepClear)
        || player_actions.just_pressed(PlayerAction::ClearZoning)
    {
        if let CurrentSelection::Structure(structure_entity) = *current_selection {
            commands
                .entity(structure_entity)
                .insert(MarkedForDemolition);
        }
    }
}

/// Spawn and despawn ghosts and apply other markings based on zoning.
fn mark_based_on_zoning(
    mut terrain_query: Query<(Entity, &mut Zoning, &TilePos, &Id<Terrain>), Changed<Zoning>>,
    structure_manifest: Res<StructureManifest>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, mut zoning, &tile_pos, &terrain) in terrain_query.iter_mut() {
        // Reborrowing here would trigger change detection, causing this system to constantly check
        match zoning.bypass_change_detection() {
            Zoning::Structure(clipboard_data) => {
                let structure_data = structure_manifest.get(clipboard_data.structure_id);
                if structure_data.allowed_terrain_types().contains(&terrain) {
                    commands.spawn_ghost(tile_pos, clipboard_data.clone())
                } else {
                    *zoning = Zoning::None;
                    // We bypassed change detection above, so need to manually trigger it here.
                    zoning.set_changed();
                }
            }
            Zoning::Terraform(mark) => {
                commands.entity(terrain_entity).insert(*mark);
            }
            Zoning::None => commands.despawn_ghost(tile_pos),
            Zoning::KeepClear => {
                commands.despawn_ghost(tile_pos);
                if let Some(structure_entity) = map_geometry.get_structure(tile_pos) {
                    commands
                        .entity(structure_entity)
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
