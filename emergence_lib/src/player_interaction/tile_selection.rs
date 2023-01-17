//! Selecting tiles to be built on, inspected or modified

use bevy::{
    prelude::{Component, MouseButton},
    utils::HashSet,
};
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::{
    prelude::InputMap,
    user_input::{InputKind, Modifier, UserInput},
    Actionlike,
};

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
    ModifySelection,
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
                TileSelectionAction::ModifySelection,
            ),
        ])
    }
}

/// The set of tiles that is currently selected
#[derive(Component, Default)]
pub struct SelectedTiles {
    selection: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Adds or removes a tile to the selection.
    ///
    /// If it is not selected, select it.
    /// If it is already selected, remove it from the selection.
    pub fn toggle_tile(&mut self, tile_pos: TilePos) {
        if self.selection.contains(&tile_pos) {
            self.selection.insert(tile_pos);
        }
    }

    /// Toggles a selection of tiles.
    ///
    /// For each tile:
    /// - if it are not selected, select it.
    /// - if it is already selected, remove it from the selection.
    pub fn toggle_tiles(&mut self, tile_pos_collection: impl IntoIterator<Item = TilePos>) {
        tile_pos_collection
            .into_iter()
            .for_each(|tile_pos| self.toggle_tile(tile_pos));
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
}
