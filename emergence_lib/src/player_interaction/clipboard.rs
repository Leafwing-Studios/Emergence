//! The clipboard stores selected structures, to later be placed via zoning.

use bevy::{prelude::*, utils::HashMap};
use hexx::{Hex, HexIterExt};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    simulation::geometry::{Facing, MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, ghost::Preview, StructureId},
};

use super::{cursor::CursorPos, selection::CurrentSelection, InteractionSystem, PlayerAction};

/// Code and data for working with the clipboard
pub(super) struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Clipboard>()
            // We're running this before we select tiles to deliberately introduce a one-frame delay,
            // ensuring that users need to double click to clear the clipboard as well.
            .add_system(clear_clipboard.before(InteractionSystem::SelectTiles))
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
                    .label(InteractionSystem::ManagePreviews)
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct StructureData {
    /// The identity of the structure.
    pub(crate) structure_id: StructureId,
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

        let center = TilePos {
            hex: self.keys().map(|tile_pos| tile_pos.hex).center(),
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
                original_pos.right_around(Hex::ZERO)
            } else {
                item.facing.rotate_left();
                original_pos.left_around(Hex::ZERO)
            };

            new_map.insert(TilePos { hex: new_pos }, item.clone());
        }

        self.contents = new_map;
    }
}

/// Clears the clipboard when the correct actions are pressed
fn clear_clipboard(
    mut clipboard: ResMut<Clipboard>,
    current_selection: Res<CurrentSelection>,
    actions: Res<ActionState<PlayerAction>>,
) {
    if current_selection.is_empty() && actions.just_pressed(PlayerAction::Deselect) {
        clipboard.clear();
    }
}

/// Copies the selected structure(s) to the clipboard, to be placed later.
///
/// This system also handles the "pipette" functionality.
fn copy_selection(
    actions: Res<ActionState<PlayerAction>>,
    mut clipboard: ResMut<Clipboard>,
    cursor_pos: Res<CursorPos>,
    current_selection: Res<CurrentSelection>,
    structure_query: Query<(&TilePos, &StructureId, &Facing), Without<Preview>>,
    map_geometry: Res<MapGeometry>,
) {
    if actions.just_pressed(PlayerAction::Pipette) {
        // We want to replace our selection, rather than add to it
        clipboard.clear();

        match &*current_selection {
            CurrentSelection::Structure(structure_entity) => {
                let (tile_pos, id, facing) = structure_query.get(*structure_entity).unwrap();
                let clipboard_item = StructureData {
                    structure_id: *id,
                    facing: *facing,
                };

                clipboard.insert(*tile_pos, clipboard_item);
                clipboard.normalize_positions();
            }
            CurrentSelection::Terrain(selected_tiles) => {
                // If there is no selection, just grab whatever's under the cursor
                if selected_tiles.is_empty() {
                } else {
                    for selected_tile_pos in selected_tiles.selection().iter() {
                        if let Some(structure_entity) =
                            map_geometry.structure_index.get(selected_tile_pos)
                        {
                            let (_tile_pos, id, facing) =
                                structure_query.get(*structure_entity).unwrap();
                            debug_assert_eq!(_tile_pos, selected_tile_pos);
                            let clipboard_item = StructureData {
                                structure_id: *id,
                                facing: *facing,
                            };

                            clipboard.insert(*selected_tile_pos, clipboard_item);
                        }
                    }
                    clipboard.normalize_positions();
                }
            }
            // Otherwise, just grab whatever's under the cursor
            _ => {
                if let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() {
                    if let Some(structure_entity) =
                        map_geometry.structure_index.get(&cursor_tile_pos)
                    {
                        let (_tile_pos, id, facing) =
                            structure_query.get(*structure_entity).unwrap();
                        debug_assert_eq!(*_tile_pos, cursor_tile_pos);
                        let clipboard_item = StructureData {
                            structure_id: *id,
                            facing: *facing,
                        };

                        clipboard.insert(TilePos::default(), clipboard_item);
                    }
                }
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
    preview_query: Query<(&TilePos, &StructureId, &Facing), With<Preview>>,
) {
    if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        let mut desired_previews: HashMap<TilePos, StructureData> =
            HashMap::with_capacity(clipboard.capacity());
        for (&clipboard_pos, clipboard_item) in clipboard.iter() {
            let tile_pos = cursor_pos + clipboard_pos;
            desired_previews.insert(tile_pos, clipboard_item.clone());
        }

        // Handle previews that already exist
        for (tile_pos, existing_structure_id, existing_facing) in preview_query.iter() {
            // Preview should exist
            if let Some(desired_clipboard_item) = desired_previews.get(tile_pos) {
                // Preview's identity changed
                if *existing_structure_id != desired_clipboard_item.structure_id
                    || *existing_facing != desired_clipboard_item.facing
                {
                    commands.despawn_preview(*tile_pos);
                } else {
                    // This ghost is still correct
                    desired_previews.remove(tile_pos);
                }

            // Preview should no longer exist
            } else {
                commands.despawn_preview(*tile_pos);
            }
        }

        // Handle any remaining new ghosts
        for (&tile_pos, clipboard_item) in desired_previews.iter() {
            commands.spawn_preview(tile_pos, clipboard_item.clone());
        }
    }
}
