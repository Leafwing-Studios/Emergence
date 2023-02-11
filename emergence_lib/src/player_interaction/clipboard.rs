//! The clipboard stores selected structures, to later be placed via zoning.

use bevy::{prelude::*, utils::HashMap};
use hexx::Hex;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    simulation::geometry::{Facing, MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, ghost::Ghost, StructureId},
};

use super::{cursor::CursorPos, tile_selection::SelectedTiles, InteractionSystem, PlayerAction};

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
                rotate_selection
                    .label(InteractionSystem::SetClipboard)
                    .after(copy_selection),
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
pub(crate) struct Clipboard {
    /// The internal map of structures.
    contents: HashMap<TilePos, StructureData>,
}

impl Clipboard {
    /// Sets the contents of the clipboard to a single structure (or clears it if [`None`] is provided).
    pub(crate) fn set(&mut self, maybe_structure: Option<StructureData>) {
        self.contents.clear();

        if let Some(structure) = maybe_structure {
            self.contents.insert(TilePos::default(), structure);
        }
    }
}

/// The data copied via the clipboard for a single structure.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct StructureData {
    /// The identity of the structure.
    pub(crate) id: StructureId,
    /// The orientation of the structure.
    pub(crate) facing: Facing,
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
    pub(super) fn offset_positions(&self, origin: TilePos) -> Vec<(TilePos, StructureData)> {
        self.iter()
            .map(|(k, v)| ((*k + origin), v.clone()))
            .collect()
    }

    /// Rotates the contents of the clipboard around the `center`.
    ///
    /// You must ensure that the contents are normalized first.
    fn rotate_around(&mut self, clockwise: bool) {
        let mut new_map = HashMap::with_capacity(self.capacity());

        for (&original_pos, item) in self.iter_mut() {
            let new_pos = if clockwise {
                item.facing.rotate_right();
                original_pos.rotate_right_around(Hex::ZERO)
            } else {
                item.facing.rotate_left();
                original_pos.rotate_left_around(Hex::ZERO)
            };

            new_map.insert(TilePos { hex: new_pos }, item.clone());
        }

        self.contents = new_map;
    }
}

/// Copies the selected structure(s) to the clipboard, to be placed later.
///
/// This system also handles the "pipette" functionality.
fn copy_selection(
    cursor: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    mut clipboard: ResMut<Clipboard>,
    selected_tiles: Res<SelectedTiles>,
    structure_query: Query<(&StructureId, &Facing), Without<Ghost>>,
    map_geometry: Res<MapGeometry>,
) {
    if actions.pressed(PlayerAction::ClearClipboard) {
        clipboard.clear();
        // Don't try to clear and set the clipboard on the same frame.
        return;
    }

    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        if actions.just_pressed(PlayerAction::Pipette) {
            // We want to replace our selection, rather than add to it
            clipboard.clear();

            // If there is no selection, just grab whatever's under the cursor
            if selected_tiles.is_empty() {
                if let Some(structure_entity) = map_geometry.structure_index.get(&cursor_tile_pos) {
                    let (id, facing) = structure_query.get(*structure_entity).unwrap();
                    let clipboard_item = StructureData {
                        id: id.clone(),
                        facing: *facing,
                    };

                    clipboard.insert(TilePos::default(), clipboard_item);
                }
            } else {
                for selected_tile_pos in selected_tiles.selection().iter() {
                    if let Some(structure_entity) =
                        map_geometry.structure_index.get(selected_tile_pos)
                    {
                        let (id, facing) = structure_query.get(*structure_entity).unwrap();
                        let clipboard_item = StructureData {
                            id: id.clone(),
                            facing: *facing,
                        };

                        clipboard.insert(*selected_tile_pos, clipboard_item);
                    }
                }
                clipboard.normalize_positions();
            }
        }
    }
}

/// Rotates the contents of the clipboard based on player input
fn rotate_selection(actions: Res<ActionState<PlayerAction>>, mut clipboard: ResMut<Clipboard>) {
    if actions.just_pressed(PlayerAction::RotateClipboardLeft)
        && actions.just_pressed(PlayerAction::RotateClipboardRight)
    {
        return;
    }

    if actions.just_pressed(PlayerAction::RotateClipboardLeft) {
        clipboard.rotate_around(false);
    }

    if actions.just_pressed(PlayerAction::RotateClipboardRight) {
        clipboard.rotate_around(true);
    }
}

/// Show the current selection under the cursor
fn display_selection(
    clipboard: Res<Clipboard>,
    cursor_pos: Res<CursorPos>,
    mut commands: Commands,
    mut ghost_query: Query<(&TilePos, &mut StructureId, &mut Facing), With<Ghost>>,
) {
    if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        let mut desired_ghosts: HashMap<TilePos, StructureData> =
            HashMap::with_capacity(clipboard.capacity());
        for (&clipboard_pos, clipboard_item) in clipboard.iter() {
            let tile_pos = cursor_pos + clipboard_pos;
            desired_ghosts.insert(tile_pos, clipboard_item.clone());
        }

        // Handle ghosts that already exist
        for (tile_pos, mut existing_structure_id, mut existing_facing) in ghost_query.iter_mut() {
            // Ghost should exist
            if let Some(desired_clipboard_item) = desired_ghosts.get(tile_pos) {
                let desired_structure_id = &desired_clipboard_item.id;
                let desired_facing = desired_clipboard_item.facing;

                // Ghost's identity changed
                if *existing_structure_id != *desired_structure_id {
                    // TODO: Bevy 0.10, use set_if_neq
                    *existing_structure_id = desired_structure_id.clone();
                }

                // Ghost's facing has changed
                if *existing_facing != desired_facing {
                    // TODO: Bevy 0.10, use set_if_neq
                    *existing_facing = desired_facing;
                }

                // This ghost has been handled
                desired_ghosts.remove(tile_pos);
            // Ghost should no longer exist
            } else {
                commands.despawn_ghost(*tile_pos);
            }
        }

        // Handle any remaining new ghosts
        for (&tile_pos, clipboard_item) in desired_ghosts.iter() {
            commands.spawn_ghost(tile_pos, clipboard_item.clone());
        }
    }
}
