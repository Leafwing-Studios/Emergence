//! Selecting tiles to be built on, inspected or modified

use bevy::{
    prelude::{App, Color, Commands, Entity, MouseButton, Plugin, Query, Res, ResMut, Resource},
    utils::HashSet,
};
use bevy_ecs_tilemap::tiles::{TileColor, TilePos};
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
#[derive(Resource, Debug, Default)]
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
            self.selection.remove(&tile_pos);
        } else {
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

    /// Does the selection contain `tile_pos`?
    pub fn contains(&self, tile_pos: &TilePos) -> bool {
        self.selection.contains(tile_pos)
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
            .add_system(select_single_tile)
            .add_system(display_selected_tiles);
    }
}

fn select_single_tile(
    cursor_tile_pos: Res<CursorTilePos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<TileSelectionAction>>,
) {
    if let Some(cursor_tile) = cursor_tile_pos.maybe_tile_pos() {
        if actions.pressed(TileSelectionAction::ModifySelection) {
            selected_tiles.toggle_tile(cursor_tile);
        } else if actions.pressed(TileSelectionAction::Single) {
            selected_tiles.clear_selection();
            selected_tiles.toggle_tile(cursor_tile);
        }
    }
}

fn display_selected_tiles(
    selected_tiles: Res<SelectedTiles>,
    mut tile_query: Query<(&mut TileColor, &TilePos)>,
) {
    if selected_tiles.is_changed() {
        for (mut tile_color, tile_pos) in tile_query.iter_mut() {
            *tile_color = match selected_tiles.contains(tile_pos) {
                true => TileColor(Color::YELLOW),
                false => TileColor(Color::WHITE),
            };
        }
    }
}
