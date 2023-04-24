//! Zoning is used to indicate that a tile should contain the specified structure.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::{manifest::Id, AssetState},
    construction::{demolition::MarkedForDemolition, ghosts::Preview},
    player_interaction::{
        clipboard::{ClipboardData, Tool},
        picking::CursorPos,
        selection::CurrentSelection,
        InteractionSystem, PlayerAction,
    },
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{MapGeometry, TilePos},
    structures::{
        commands::StructureCommandsExt,
        structure_manifest::{Structure, StructureManifest},
    },
    terrain::{
        commands::TerrainCommandsExt,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

use super::terraform::TerraformingAction;

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
        .add_system(cleanup_previews.after(set_zoning))
        .add_system(
            mark_based_on_zoning
                .in_set(InteractionSystem::ManagePreviews)
                .run_if(in_state(AssetState::FullyLoaded))
                .after(InteractionSystem::ApplyZoning),
        )
        // Must run after crafting emitters in order to wipe out their signals
        .add_system(keep_tiles_clear.after(crate::crafting::set_crafting_emitter));
    }
}

/// The zoning of a given tile, which specifies which structure *should* be built there.
#[derive(Component, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Zoning {
    /// The provided structure should be built on this tile.
    Structure(ClipboardData),
    /// The provided terraforming should be applied to this tile.
    Terraform(TerraformingAction),
    /// No zoning is set.
    None,
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
            Zoning::Terraform(action) => action.display(terrain_manifest),
            Zoning::None => "None".to_string(),
        }
    }
}

/// Cleans up all old previews.
///
/// We're just using an immediate mode system for this, since it's much easier to ensure correctness.
fn cleanup_previews(
    mut commands: Commands,
    new_query: Query<Entity, (With<Preview>, Without<CleanMeUp>)>,
    old_query: Query<Entity, (With<Preview>, With<CleanMeUp>)>,
) {
    for entity in new_query.iter() {
        commands.entity(entity).insert(CleanMeUp);
    }

    for entity in old_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// A marker component for previews that should be deleted.
#[derive(Component)]
struct CleanMeUp;

/// Applies zoning to an area, causing structures to be created (or removed) there.
///
/// This system also displays previews in order to ensure perfect consistency.
fn set_zoning(
    cursor_pos: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    tool: Res<Tool>,
    mut zoning_query: Query<&mut Zoning>,
    terrain_preview_query: Query<(&Id<Terrain>, &TilePos)>,
    current_selection: Res<CurrentSelection>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
    let relevant_terrain_entities = relevant_tiles.entities(&map_geometry);

    // Explicitly clear the selection
    if actions.pressed(PlayerAction::ClearZoning) {
        for terrain_entity in relevant_terrain_entities {
            let mut zoning = zoning_query.get_mut(terrain_entity).unwrap();
            *zoning = Zoning::None;
        }

        // Don't try to clear and zone in the same frame
        return;
    }

    // Apply zoning
    let apply_zoning = actions.pressed(PlayerAction::Paste)
        || actions.pressed(PlayerAction::UseTool) && !tool.is_empty();

    match &*tool {
        Tool::Terraform(terraform_tool) => match apply_zoning {
            true => {
                for terrain_entity in relevant_terrain_entities {
                    let mut zoning = zoning_query.get_mut(terrain_entity).unwrap();
                    *zoning = Zoning::Terraform((*terraform_tool).into());
                }
            }
            false => {
                for terrain_entity in relevant_terrain_entities {
                    let (&terrain_id, &tile_pos) =
                        terrain_preview_query.get(terrain_entity).unwrap();
                    let terraforming_action = TerraformingAction::from(*terraform_tool);

                    commands.spawn_preview_terrain(tile_pos, terrain_id, terraforming_action);
                }
            }
        },
        Tool::Structures(map) => {
            // Zone using the single selected structure
            match map.len() {
                0 => (),
                1 => {
                    let clipboard_item = map.values().next().unwrap();
                    match apply_zoning {
                        true => {
                            for terrain_entity in relevant_terrain_entities {
                                let mut zoning = zoning_query.get_mut(terrain_entity).unwrap();
                                *zoning = Zoning::Structure(clipboard_item.clone());
                            }
                        }
                        false => {
                            for &tile_pos in relevant_tiles.selection().iter() {
                                commands.spawn_preview_structure(tile_pos, clipboard_item.clone());
                            }
                        }
                    }
                }
                _ => {
                    let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() else {
                        return;
                    };

                    for (tile_pos, clipboard_item) in tool.offset_positions(cursor_tile_pos) {
                        match apply_zoning {
                            true => {
                                // Avoid trying to operate on terrain that doesn't exist
                                if let Some(terrain_entity) = map_geometry.get_terrain(tile_pos) {
                                    let mut zoning = zoning_query.get_mut(terrain_entity).unwrap();
                                    *zoning = Zoning::Structure(clipboard_item.clone());
                                }
                            }
                            false => {
                                commands.spawn_preview_structure(tile_pos, clipboard_item);
                            }
                        }
                    }
                }
            }
        }
        Tool::Ability(..) => (),
        Tool::None => (),
    }
}

/// Mark the selected structure for deletion.
fn mark_for_demolition(
    player_actions: Res<ActionState<PlayerAction>>,
    current_selection: Res<CurrentSelection>,
    mut commands: Commands,
) {
    if player_actions.just_pressed(PlayerAction::ClearZoning) {
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
    for (terrain_entity, mut zoning, &tile_pos, current_terrain_id) in terrain_query.iter_mut() {
        // Reborrowing here would trigger change detection, causing this system to constantly check
        match zoning.bypass_change_detection() {
            Zoning::Structure(clipboard_data) => {
                let footprint =
                    structure_manifest.construction_footprint(clipboard_data.structure_id);

                if map_geometry.can_build(tile_pos, &footprint.rotated(clipboard_data.facing)) {
                    commands.spawn_ghost_structure(tile_pos, clipboard_data.clone())
                } else {
                    *zoning = Zoning::None;
                    // We bypassed change detection above, so need to manually trigger it here.
                    zoning.set_changed();
                }
            }
            Zoning::Terraform(terraforming_action) => {
                commands.entity(terrain_entity).insert(*terraforming_action);

                // We neeed to use the model for the terrain we're changing to, not the current one
                let terrain_id = match terraforming_action {
                    TerraformingAction::Change(terrain_id) => *terrain_id,
                    _ => *current_terrain_id,
                };

                commands.spawn_ghost_terrain(tile_pos, terrain_id, *terraforming_action);

                // Mark any structures that are here as needing to be demolished
                // Terraforming can't be done with roots growing into stuff!
                if let Some(structure_entity) = map_geometry.get_structure(tile_pos) {
                    commands
                        .entity(structure_entity)
                        .insert(MarkedForDemolition);
                }
            }
            Zoning::None => {
                commands.despawn_ghost_terrain(tile_pos);
                commands.despawn_ghost_structure(tile_pos);
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
