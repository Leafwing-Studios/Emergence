use bevy::{prelude::*, utils::HashMap};
use hexx::Hex;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    simulation::geometry::{MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, ghost::Ghost, StructureId},
};

use super::{cursor::CursorPos, tile_selection::SelectedTiles, InteractionSystem, SelectionAction};

/// Code and data for working with the clipboard
pub(super) struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Clipboard>()
            .add_system(
                copy_selection
                    .label(InteractionSystem::SetClipboard)
                    .after(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::SelectTiles),
            )
            .add_system(
                display_selection
                    .label(InteractionSystem::ManageGhosts)
                    .after(InteractionSystem::SetClipboard),
            );
    }
}

/// Stores a selection to copy and paste.
#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub(super) struct Clipboard {
    /// The internal map of structures.
    contents: HashMap<TilePos, StructureId>,
}

impl Clipboard {
    /// Normalizes the positions of the items on the clipboard.
    ///
    /// Centers relative to the median selected tile position.
    /// Each axis is computed independently.
    fn normalize_positions(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut x_vec = Vec::from_iter(self.keys().map(|tile_pos| tile_pos.x));
        let mut y_vec = Vec::from_iter(self.keys().map(|tile_pos| tile_pos.y));

        x_vec.sort_unstable();
        y_vec.sort_unstable();

        let mid = self.len() / 2;
        let center = TilePos {
            hex: Hex {
                x: x_vec[mid],
                y: y_vec[mid],
            },
        };

        let mut new_map = HashMap::with_capacity(self.capacity());

        for (tile_pos, id) in self.iter() {
            let new_tile_pos = *tile_pos - center;
            // PERF: eh maybe we can safe a clone by using remove?
            new_map.insert(new_tile_pos, id.clone());
        }

        self.contents = new_map;
    }

    /// Apply a tile-position shift to the items on the clipboard.
    ///
    /// Used to place items in the correct location relative to the cursor.
    pub(super) fn offset_positions(&self, origin: TilePos) -> Vec<(TilePos, StructureId)> {
        self.iter()
            .map(|(k, v)| ((*k + origin), v.clone()))
            .collect()
    }
}

/// Copies the selected structure(s) to the clipboard, to be placed later.
///
/// This system also handles the "pipette" functionality.
fn copy_selection(
    cursor: Res<CursorPos>,
    actions: Res<ActionState<SelectionAction>>,
    mut clipboard: ResMut<Clipboard>,
    selected_tiles: Res<SelectedTiles>,
    structure_query: Query<&StructureId>,
    map_geometry: Res<MapGeometry>,
) {
    if actions.pressed(SelectionAction::ClearClipboard) {
        clipboard.clear();
        // Don't try to clear and set the clipboard on the same frame.
        return;
    }

    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        if actions.just_pressed(SelectionAction::Pipette) {
            // We want to replace our selection, rather than add to it
            clipboard.clear();

            // If there is no selection, just grab whatever's under the cursor
            if selected_tiles.is_empty() {
                if let Some(structure_entity) = map_geometry.structure_index.get(&cursor_tile_pos) {
                    let structure_id = structure_query.get(*structure_entity).unwrap();
                    clipboard.insert(TilePos::default(), structure_id.clone());
                }
            } else {
                for selected_tile_pos in selected_tiles.selection().iter() {
                    if let Some(structure_entity) =
                        map_geometry.structure_index.get(selected_tile_pos)
                    {
                        let structure_id = structure_query.get(*structure_entity).unwrap();
                        clipboard.insert(*selected_tile_pos, structure_id.clone());
                    }
                }
                clipboard.normalize_positions();
            }
        }
    }
}

/// Show the current selection under the cursor
fn display_selection(
    clipboard: Res<Clipboard>,
    cursor_pos: Res<CursorPos>,
    mut commands: Commands,
    mut ghost_query: Query<(&TilePos, &mut StructureId), With<Ghost>>,
) {
    if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        let mut desired_ghosts: HashMap<TilePos, StructureId> =
            HashMap::with_capacity(clipboard.capacity());
        for (&clipboard_pos, structure_id) in clipboard.iter() {
            let tile_pos = cursor_pos + clipboard_pos;
            desired_ghosts.insert(tile_pos, structure_id.clone());
        }

        // Handle ghosts that already exist
        for (tile_pos, mut existing_structure_id) in ghost_query.iter_mut() {
            // Ghost should exist
            if let Some(desired_structure_id) = desired_ghosts.get(tile_pos) {
                // Ghost's identity changed
                if *existing_structure_id != *desired_structure_id {
                    // TODO: Bevy 0.10, use set_if_neq
                    *existing_structure_id = desired_structure_id.clone();
                }

                // This ghost has been handled
                desired_ghosts.remove(tile_pos);
            // Ghost should no longer exist
            } else {
                commands.despawn_ghost(*tile_pos);
            }
        }

        // Handle any remaining new ghosts
        for (&tile_pos, id) in desired_ghosts.iter() {
            commands.spawn_ghost(tile_pos, id.clone());
        }
    }
}
