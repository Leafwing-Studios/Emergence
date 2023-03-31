//! The clipboard stores selected structures, to later be placed via zoning.

use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashMap};
use hexx::{Hex, HexIterExt};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Facing, MapGeometry, TilePos},
    structures::{construction::Preview, crafting::ActiveRecipe, structure_manifest::Structure},
};

use super::{
    cursor::CursorPos, selection::CurrentSelection, terraform::TerraformingChoice,
    InteractionSystem, PlayerAction,
};

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
                    .in_set(InteractionSystem::SetClipboard)
                    .after(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::SelectTiles),
            )
            .add_system(
                rotate_selection
                    .in_set(InteractionSystem::SetClipboard)
                    .after(copy_selection),
            );
    }
}

/// Stores a selection to copy and paste.
#[derive(Default, Resource, Debug)]
pub(crate) enum Clipboard {
    /// The clipboard is set to terraform terrain.
    Terraform(TerraformingChoice),
    /// The clipboard contains a structure.
    Structures(HashMap<TilePos, ClipboardData>),
    /// The clipboard is empty.
    #[default]
    Empty,
}

impl Clipboard {
    /// Is the clipboard empty?
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Clipboard::Empty => true,
            Clipboard::Structures(map) => map.is_empty(),
            Clipboard::Terraform(_) => false,
        }
    }

    /// Sets the contents of the clipboard to a single structure (or clears it if [`None`] is provided).
    pub(crate) fn set_to_structure(&mut self, maybe_structure: Option<ClipboardData>) {
        *self = match maybe_structure {
            Some(clipboard_data) => Clipboard::Structures({
                let mut map = HashMap::new();
                map.insert(TilePos::default(), clipboard_data);
                map
            }),
            None => Clipboard::Empty,
        };
    }
}

/// The data copied via the clipboard for a single structure.
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ClipboardData {
    /// The identity of the structure.
    pub(crate) structure_id: Id<Structure>,
    /// The orientation of the structure.
    pub(crate) facing: Facing,
    /// The recipe that this structure makes, if any
    pub(crate) active_recipe: ActiveRecipe,
}

impl Clipboard {
    /// Normalizes the positions of the items on the clipboard.
    ///
    /// Centers relative to the median selected tile position.
    /// Each axis is computed independently.
    fn normalize_positions(&mut self) {
        if let Clipboard::Structures(map) = self {
            let center = TilePos {
                hex: map.keys().map(|tile_pos| tile_pos.hex).center(),
            };

            let mut new_map = HashMap::with_capacity(map.capacity());

            for (tile_pos, id) in map.iter() {
                let new_tile_pos = *tile_pos - center;
                // PERF: eh maybe we can safe a clone by using remove?
                new_map.insert(new_tile_pos, id.clone());
            }

            *map = new_map;
        }
    }

    /// Apply a tile-position shift to the items on the clipboard.
    ///
    /// Used to place items in the correct location relative to the cursor.
    pub(super) fn offset_positions(&self, origin: TilePos) -> Vec<(TilePos, ClipboardData)> {
        if let Clipboard::Structures(map) = self {
            map.iter()
                .map(|(k, v)| ((*k + origin), v.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Rotates the contents of the clipboard around the `center`.
    ///
    /// You must ensure that the contents are normalized first.
    fn rotate_around(&mut self, clockwise: bool) {
        if let Clipboard::Structures(map) = self {
            let mut new_map = HashMap::with_capacity(map.capacity());

            for (&original_pos, item) in map.iter_mut() {
                let new_pos = if clockwise {
                    item.facing.rotate_right();
                    original_pos.right_around(Hex::ZERO)
                } else {
                    item.facing.rotate_left();
                    original_pos.left_around(Hex::ZERO)
                };

                new_map.insert(TilePos { hex: new_pos }, item.clone());
            }

            *map = new_map;
        }
    }
}

/// Clears the clipboard when the correct actions are pressed
fn clear_clipboard(mut clipboard: ResMut<Clipboard>, actions: Res<ActionState<PlayerAction>>) {
    if actions.just_pressed(PlayerAction::Deselect) {
        *clipboard = Clipboard::Empty;
    }
}

/// Data needed for [`copy_selection`] to populate [`ClipboardData`].
#[derive(WorldQuery)]
struct ClipboardQuery {
    /// The position of the structure
    tile_pos: &'static TilePos,
    /// The type of the structure
    structure_id: &'static Id<Structure>,
    /// The direction the structure is facing
    facing: &'static Facing,
    /// The recipe that the structure is crafting, if any
    active_recipe: Option<&'static ActiveRecipe>,
}

impl From<ClipboardQueryItem<'_>> for ClipboardData {
    fn from(value: ClipboardQueryItem) -> ClipboardData {
        let active_recipe = match value.active_recipe {
            Some(recipe) => recipe.clone(),
            None => ActiveRecipe::default(),
        };

        ClipboardData {
            structure_id: *value.structure_id,
            facing: *value.facing,
            active_recipe,
        }
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
    structure_query: Query<ClipboardQuery, Without<Preview>>,
    map_geometry: Res<MapGeometry>,
) {
    if actions.just_pressed(PlayerAction::Copy) {
        // We want to replace our selection, rather than add to it
        let mut map = HashMap::new();

        match &*current_selection {
            CurrentSelection::Structure(entity) | CurrentSelection::Ghost(entity) => {
                let query_item = structure_query.get(*entity).unwrap();
                let tile_pos = query_item.tile_pos;
                let clipboard_data = query_item.into();
                map.insert(*tile_pos, clipboard_data);
                *clipboard = Clipboard::Structures(map);
                clipboard.normalize_positions();
            }
            CurrentSelection::Terrain(selected_tiles) => {
                // If there is no selection, just grab whatever's under the cursor
                if selected_tiles.is_empty() {
                    if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                        if let Some(entity) = map_geometry.get_ghost_or_structure(hovered_tile) {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(TilePos::default(), clipboard_data);
                        }
                    }
                } else {
                    for &selected_tile_pos in selected_tiles.selection().iter() {
                        if let Some(entity) = map_geometry.get_ghost_or_structure(selected_tile_pos)
                        {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(selected_tile_pos, clipboard_data);
                        }
                    }
                    *clipboard = Clipboard::Structures(map);
                    clipboard.normalize_positions();
                }
            }
            // Otherwise, just grab whatever's under the cursor
            CurrentSelection::None | CurrentSelection::Unit(_) => {
                if let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() {
                    if let Some(structure_entity) = map_geometry.get_structure(cursor_tile_pos) {
                        let clipboard_data = structure_query.get(structure_entity).unwrap().into();
                        map.insert(TilePos::default(), clipboard_data);
                        *clipboard = Clipboard::Structures(map);
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
