//! Zoning is used to indicate that a tile should contain the specified structure.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::Id,
    construction::{demolition::MarkedForDemolition, ghosts::Preview},
    geometry::MapGeometry,
    player_interaction::{
        clipboard::Tool, picking::CursorPos, selection::CurrentSelection, InteractionSystem,
        PlayerAction, PlayerModifiesWorld,
    },
    structures::{commands::StructureCommandsExt, structure_manifest::Structure, Landmark},
};

use super::terraform::TerraformingCommandsExt;

/// Code and data for setting zoning of areas for construction.
pub(super) struct ZoningPlugin;

impl Plugin for ZoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (mark_for_demolition, set_zoning)
                .in_set(InteractionSystem::ApplyZoning)
                .in_set(PlayerModifiesWorld)
                .after(InteractionSystem::SelectTiles)
                .after(InteractionSystem::SetClipboard),
        )
        .add_system(cleanup_previews.after(set_zoning));
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
    current_selection: Res<CurrentSelection>,
    mut commands: Commands,
) {
    let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);

    // Explicitly clear the selection
    if actions.pressed(PlayerAction::ClearZoning) {
        for &voxel_pos in relevant_tiles.iter() {
            commands.despawn_ghost_structure(voxel_pos);
            commands.cancel_terraform(voxel_pos.hex);
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
                for voxel_pos in relevant_tiles.iter() {
                    commands.start_terraform(voxel_pos.hex, terraform_tool.clone().into());
                }
            }
            false => {
                for &voxel_pos in relevant_tiles.iter() {
                    commands.preview_terraform(voxel_pos.hex, terraform_tool.clone().into());
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
                            for voxel_pos in relevant_tiles.iter() {
                                commands.spawn_ghost_structure(*voxel_pos, clipboard_item.clone());
                            }
                        }
                        false => {
                            for voxel_pos in relevant_tiles.iter() {
                                commands
                                    .spawn_preview_structure(*voxel_pos, clipboard_item.clone());
                            }
                        }
                    }
                }
                _ => {
                    let Some(cursor_tile_pos) = cursor_pos.maybe_voxel_pos() else {
                        return;
                    };

                    for (voxel_pos, clipboard_item) in tool.offset_positions(cursor_tile_pos) {
                        match apply_zoning {
                            true => {
                                commands.spawn_ghost_structure(voxel_pos, clipboard_item.clone());
                            }
                            false => {
                                commands.spawn_preview_structure(voxel_pos, clipboard_item.clone());
                            }
                        }
                    }
                }
            }
        }
        Tool::None => (),
    }
}

/// Mark the selected structure for deletion.
fn mark_for_demolition(
    player_actions: Res<ActionState<PlayerAction>>,
    current_selection: Res<CurrentSelection>,
    // Landmarks can't be demolished
    structure_query: Query<&Id<Structure>, Without<Landmark>>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    if player_actions.just_pressed(PlayerAction::ClearZoning) {
        if let CurrentSelection::Voxels(ref selected_voxels) = *current_selection {
            for voxel_object in selected_voxels.voxel_objects(&map_geometry) {
                if structure_query.contains(voxel_object.entity) {
                    commands
                        .entity(voxel_object.entity)
                        .insert(MarkedForDemolition);
                }
            }
        }
    }
}
