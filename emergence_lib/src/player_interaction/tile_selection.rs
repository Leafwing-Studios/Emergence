//! Selecting tiles to be built on, inspected or modified

use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::tiles::{TilePos, TileVisible};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::{InputKind, Modifier, UserInput},
    Actionlike,
};

use super::cursor::CursorTilePos;

/// Actions that can be used to select tiles.
///
/// If a tile is not selected, it will be added to the selection.
/// If it is already selected, it will be removed from the selection.
#[derive(Actionlike, Clone)]
pub enum TileSelectionAction {
    /// Selects a single tile.
    ///
    /// This behavior is performed by `Click` in most applications.
    Single,
    /// Adds additional tiles to the selection
    ///
    /// This behavior is performed by `Control + Click` in most applications.
    Modify,
    /// Selects or deselects a group of hex tiles by dragging over them
    Drag,
}

#[derive(PartialEq, Default)]
enum SelectMode {
    #[default]
    None,
    Select,
    Deselect,
}

impl TileSelectionAction {
    pub(super) fn default_input_map() -> InputMap<TileSelectionAction> {
        InputMap::new([
            (
                UserInput::Single(InputKind::Mouse(MouseButton::Left)),
                TileSelectionAction::Single,
            ),
            (
                UserInput::modified(Modifier::Control, MouseButton::Left),
                TileSelectionAction::Modify,
            ),
            (
                UserInput::modified(Modifier::Shift, MouseButton::Left),
                TileSelectionAction::Drag,
            ),
        ])
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default)]
pub struct SelectedTiles {
    selection: HashSet<TilePos>,
    // TODO: use this for more efficient tile selection toggling
    previous_selection: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    pub fn select_tile(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        self.selection.insert(tile_pos);
    }

    /// Deselects a single tile
    pub fn deselect_tile(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        self.selection.remove(&tile_pos);
    }

    /// Selects or unselects a single tile.
    ///
    /// If it is not selected, select it.
    /// If it is already selected, remove it from the selection.
    pub fn toggle_tile(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        if self.selection.contains(&tile_pos) {
            self.selection.clear();
        } else {
            // Clear and then reinsert rather than making a new struct
            // to avoid a pointless allocation
            self.selection.clear();
            self.selection.insert(tile_pos);
        }
    }

    /// Toggles the selection of a group of tiles.
    ///
    /// For each tile:
    /// - if it are not selected, select it.
    /// - if it is already selected, remove it from the selection.
    pub fn toggle_tiles(&mut self, tile_pos_collection: impl IntoIterator<Item = TilePos>) {
        self.cache_selection();

        tile_pos_collection.into_iter().for_each(|tile_pos| {
            if self.selection.contains(&tile_pos) {
                self.selection.remove(&tile_pos);
            } else {
                self.selection.insert(tile_pos);
            }
        });
    }

    /// Adds or removes a tile to the selection.
    ///
    /// If it is not selected, select it.
    /// If it is already selected, remove it from the selection.
    pub fn modify_selection(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        if self.selection.contains(&tile_pos) {
            self.selection.remove(&tile_pos);
        } else {
            self.selection.insert(tile_pos);
        }
    }

    /// The current set of selected tiles
    pub fn selection(&self) -> &HashSet<TilePos> {
        &self.selection
    }

    /// The previous set of selected tiles
    pub fn previous_selection(&self) -> &HashSet<TilePos> {
        &self.previous_selection
    }

    /// Stores the current tile selection to be used to compute the set of changed tiles efficiently
    fn cache_selection(&mut self) {
        self.previous_selection = self.selection.clone();
    }

    /// Clears the set of selected tiles.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// The number of selected tiles.
    pub fn len(&self) -> usize {
        self.selection.len()
    }

    /// Are any tiles selected?
    pub fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }

    /// Is the given tile in the selection?
    pub fn contains(&self, tile_pos: &TilePos) -> bool {
        self.selection.contains(&tile_pos)
    }

    /// Compute the set of newly added tiles
    pub fn added_tiles(&self) -> HashSet<TilePos> {
        self.selection
            .difference(self.previous_selection())
            .into_iter()
            .map(|p| *p)
            .collect()
    }

    /// Compute the set of newly removed tiles
    pub fn removed_tiles(&self) -> HashSet<TilePos> {
        self.previous_selection
            .difference(self.selection())
            .into_iter()
            .map(|p| *p)
            .collect()
    }
}

/// All tile selection logic and graphics
pub(super) struct TileSelectionPlugin;

impl Plugin for TileSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTiles>()
            .init_resource::<ActionState<TileSelectionAction>>()
            .insert_resource(TileSelectionAction::default_input_map())
            .add_plugin(InputManagerPlugin::<TileSelectionAction>::default())
            .add_system(select_tiles)
            .add_system(display_selected_tiles.after(select_tiles));
    }
}

fn select_tiles(
    cursor_tile_pos: Res<CursorTilePos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<TileSelectionAction>>,
    mut selection_mode: Local<SelectMode>,
) {
    if let Some(cursor_tile) = cursor_tile_pos.maybe_tile_pos() {
        if actions.pressed(TileSelectionAction::Drag) {
            if *selection_mode == SelectMode::None {
                *selection_mode = match selected_tiles.contains(&cursor_tile) {
                    // If you start with a selected tile, subtract from the selection
                    true => SelectMode::Deselect,
                    // If you start with an unselected tile, add to the selection
                    false => SelectMode::Select,
                }
            }
            match *selection_mode {
                SelectMode::Select => selected_tiles.select_tile(cursor_tile),
                SelectMode::Deselect => selected_tiles.deselect_tile(cursor_tile),
                SelectMode::None => unreachable!(),
            }
        } else if actions.just_pressed(TileSelectionAction::Modify) {
            selected_tiles.modify_selection(cursor_tile);
        } else if actions.just_pressed(TileSelectionAction::Single) {
            selected_tiles.toggle_tile(cursor_tile);
        }

        if actions.released(TileSelectionAction::Drag) {
            *selection_mode = SelectMode::None;
        }
    }
}

fn display_selected_tiles(
    selected_tiles: Res<SelectedTiles>,
    mut tile_query: Query<(&mut TileVisible, &TilePos)>,
) {
    if selected_tiles.is_changed() {
        for (mut tile_visibility, tile_pos) in tile_query.iter_mut() {
            *tile_visibility = match selected_tiles.contains(tile_pos) {
                true => TileVisible(true),
                false => TileVisible(false),
            };
        }
    }
}
