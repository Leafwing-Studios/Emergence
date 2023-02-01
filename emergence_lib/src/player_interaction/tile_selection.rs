//! Selecting tiles to be built on, inspected or modified

use bevy::{prelude::*, utils::HashSet};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::{InputKind, Modifier, UserInput},
    Actionlike,
};

use crate::simulation::map::TilePos;

use super::{cursor::CursorTilePos, InteractionSystem};

/// Actions that can be used to select tiles.
///
/// If a tile is not selected, it will be added to the selection.
/// If it is already selected, it will be removed from the selection.
#[derive(Actionlike, Clone)]
pub enum TileSelectionAction {
    /// Selects a single tile, deselecting any others.
    ///
    /// If the tile is already selected, it will be unselected.
    Single,
    /// Adds or subtracts tiles from the selection.
    ///
    /// Unselected tiles will be selected, and selected tiles be unslected.
    Modify,
    /// Selects or deselects a group of hex tiles by dragging over them
    ///
    /// This action will track whether you are selecting or deselecting tiles based on the state of the first tile modified with this action.
    Multiple,
    /// Clears the entire tile selection.
    Clear,
}

/// Determines how the player input impacts a chosen tile.
#[derive(PartialEq, Default)]
enum SelectMode {
    #[default]
    /// An "empty" default state
    None,
    /// Allows the player to select a tile
    Select,
    /// Deselects an already selected tile
    Deselect,
}

impl TileSelectionAction {
    /// The default key bindings
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
                TileSelectionAction::Multiple,
            ),
        ])
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default)]
pub struct SelectedTiles {
    /// Actively selected tiles
    selection: HashSet<TilePos>,
    // TODO: use this for more efficient tile selection toggling
    /// Most recently deselected tiles
    previous_selection: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    pub fn add_tile(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        self.selection.insert(tile_pos);
    }

    /// Deselects a single tile
    pub fn remove_tile(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        self.selection.remove(&tile_pos);
    }

    /// Selects a single tile, at the expense of any other tiles already selected.
    ///
    /// If a tile is not selected, select it.
    /// If a tile is already selected, remove it from the selection.
    ///
    /// This is the behavior controlled by [`TileSelectionAction::Single`].
    pub fn select_single(&mut self, tile_pos: TilePos) {
        self.cache_selection();
        if self.selection.contains(&tile_pos) {
            self.selection.clear();
        } else {
            // Clear cache then reinsert in the previous cache structure rather than making a new one
            // to avoid a pointless reallocation
            self.selection.clear();
            self.selection.insert(tile_pos);
        }
    }

    /// Adds or removes a tile from the cached selection.
    ///
    /// If it is not selected, select it.
    /// If it is already selected, remove it from the selection.
    ///
    /// This is the behavior controlled by [`TileSelectionAction::Modify`].
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
        self.cache_selection();
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
    pub fn contains_pos(&self, tile_pos: &TilePos) -> bool {
        self.selection.contains(tile_pos)
    }

    /// Compute the set of newly added tiles
    pub fn added_tiles(&self) -> HashSet<TilePos> {
        self.selection
            .difference(self.previous_selection())
            .into_iter()
            .copied()
            .collect()
    }

    /// Compute the set of newly removed tiles
    pub fn removed_tiles(&self) -> HashSet<TilePos> {
        self.previous_selection
            .difference(self.selection())
            .into_iter()
            .copied()
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
            .add_system(
                select_tiles
                    .label(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            )
            .add_system(highlight_selected_tiles.after(InteractionSystem::SelectTiles));
    }
}

/// Integrates user input into tile selection actions to let other systems handle what happens to a selected tile
fn select_tiles(
    cursor_tile_pos: Res<CursorTilePos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<TileSelectionAction>>,
    mut selection_mode: Local<SelectMode>,
) {
    if let Some(cursor_tile) = cursor_tile_pos.maybe_tile_pos() {
        if actions.pressed(TileSelectionAction::Clear) {
            selected_tiles.clear_selection();
        } else if actions.pressed(TileSelectionAction::Multiple) {
            if *selection_mode == SelectMode::None {
                *selection_mode = match selected_tiles.contains_pos(&cursor_tile) {
                    // If you start with a selected tile, subtract from the selection
                    true => SelectMode::Deselect,
                    // If you start with an unselected tile, add to the selection
                    false => SelectMode::Select,
                }
            }
            match *selection_mode {
                SelectMode::Select => selected_tiles.add_tile(cursor_tile),
                SelectMode::Deselect => selected_tiles.remove_tile(cursor_tile),
                SelectMode::None => unreachable!(),
            }
        } else if actions.just_pressed(TileSelectionAction::Modify) {
            selected_tiles.modify_selection(cursor_tile);
        } else if actions.just_pressed(TileSelectionAction::Single) {
            selected_tiles.select_single(cursor_tile);
        }

        if actions.released(TileSelectionAction::Multiple) {
            *selection_mode = SelectMode::None;
        }
    }
}
