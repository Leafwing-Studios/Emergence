//! Selecting tiles to be built on, inspected or modified

use bevy::{prelude::*, utils::HashSet};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::{InputKind, Modifier, UserInput},
    Actionlike,
};

use crate::terrain::Terrain;

use super::{cursor::CursorPos, InteractionSystem};

/// Actions that can be used to select tiles.
///
/// If a tile is not selected, it will be added to the selection.
/// If it is already selected, it will be removed from the selection.
#[derive(Actionlike, Clone, Debug)]
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
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Escape)),
                TileSelectionAction::Clear,
            ),
        ])
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default)]
pub struct SelectedTiles {
    /// Actively selected tiles
    selection: HashSet<Entity>,
}

impl SelectedTiles {
    /// Selects a single tile
    pub fn add_tile(&mut self, tile_entity: Entity) {
        self.selection.insert(tile_entity);
    }

    /// Deselects a single tile
    pub fn remove_tile(&mut self, tile_entity: Entity) {
        self.selection.remove(&tile_entity);
    }

    /// Selects a single tile, at the expense of any other tiles already selected.
    ///
    /// If a tile is not selected, select it.
    /// If a tile is already selected, remove it from the selection.
    ///
    /// This is the behavior controlled by [`TileSelectionAction::Single`].
    pub fn select_single(&mut self, tile_entity: Entity) {
        if self.selection.contains(&tile_entity) {
            self.selection.clear();
        } else {
            // Clear cache then reinsert in the previous cache structure rather than making a new one
            // to avoid a pointless reallocation
            self.selection.clear();
            self.selection.insert(tile_entity);
        }
    }

    /// Adds or removes a tile from the cached selection.
    ///
    /// If it is not selected, select it.
    /// If it is already selected, remove it from the selection.
    ///
    /// This is the behavior controlled by [`TileSelectionAction::Modify`].
    pub fn modify_selection(&mut self, tile_entity: Entity) {
        if self.selection.contains(&tile_entity) {
            self.selection.remove(&tile_entity);
        } else {
            self.selection.insert(tile_entity);
        }
    }

    /// The current set of selected tiles
    pub fn selection(&self) -> &HashSet<Entity> {
        &self.selection
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
    pub fn contains_tile(&self, tile_entity: Entity) -> bool {
        self.selection.contains(&tile_entity)
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
    cursor_tile_entity: Res<CursorPos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<TileSelectionAction>>,
    mut selection_mode: Local<SelectMode>,
) {
    if let Some(cursor_entity) = cursor_tile_entity.maybe_entity() {
        if actions.pressed(TileSelectionAction::Clear) {
            selected_tiles.clear_selection();
        };

        if actions.pressed(TileSelectionAction::Multiple) {
            if *selection_mode == SelectMode::None {
                *selection_mode = match selected_tiles.contains_tile(cursor_entity) {
                    // If you start with a selected tile, subtract from the selection
                    true => SelectMode::Deselect,
                    // If you start with an unselected tile, add to the selection
                    false => SelectMode::Select,
                }
            }
            match *selection_mode {
                SelectMode::Select => selected_tiles.add_tile(cursor_entity),
                SelectMode::Deselect => selected_tiles.remove_tile(cursor_entity),
                SelectMode::None => unreachable!(),
            }
        } else {
            *selection_mode = SelectMode::None;
        };

        if actions.just_pressed(TileSelectionAction::Modify) {
            selected_tiles.modify_selection(cursor_entity);
        };

        if actions.just_pressed(TileSelectionAction::Single) {
            selected_tiles.select_single(cursor_entity);
        }
    }
}

/// Highlights the current set of selected tiles
pub fn highlight_selected_tiles(
    selected_tiles: Res<SelectedTiles>,
    mut terrain_query: Query<(Entity, &mut Handle<StandardMaterial>), With<Terrain>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if selected_tiles.is_changed() {
        let selection = selected_tiles.selection();
        // PERF: We should probably avoid a linear scan over all tiles here
        for (terrain_entity, mut material) in terrain_query.iter_mut() {
            if selection.contains(&terrain_entity) {
                // PERF: cache these materials
                *material = materials.add(Color::GREEN.into());
            } else {
                // FIXME: reset to the correct material
                *material = materials.add(Color::WHITE.into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedTiles;
    use bevy::ecs::entity::Entity;

    #[test]
    fn simple_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_entity = Entity::from_bits(0);

        selected_tiles.add_tile(tile_entity);
        assert!(selected_tiles.contains_tile(tile_entity));
        assert!(!selected_tiles.is_empty());
        assert_eq!(selected_tiles.selection().len(), 1);

        selected_tiles.remove_tile(tile_entity);
        assert!(!selected_tiles.contains_tile(tile_entity));
        assert!(selected_tiles.is_empty());
        assert_eq!(selected_tiles.selection().len(), 0);
    }

    #[test]
    fn multi_select() {
        let mut selected_tiles = SelectedTiles::default();
        selected_tiles.add_tile(Entity::from_bits(0));
        // Intentionally doubled
        selected_tiles.add_tile(Entity::from_bits(0));
        selected_tiles.add_tile(Entity::from_bits(1));
        selected_tiles.add_tile(Entity::from_bits(2));

        assert_eq!(selected_tiles.selection().len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_tiles = SelectedTiles::default();
        selected_tiles.add_tile(Entity::from_bits(0));
        selected_tiles.add_tile(Entity::from_bits(1));
        selected_tiles.add_tile(Entity::from_bits(2));

        assert_eq!(selected_tiles.selection().len(), 3);
        selected_tiles.clear_selection();
        assert_eq!(selected_tiles.selection().len(), 0);
    }

    #[test]
    fn select_single_not_yet_selected() {
        let mut selected_tiles = SelectedTiles::default();
        let existing_entity = Entity::from_bits(0);
        let new_entity = Entity::from_bits(1);

        selected_tiles.add_tile(existing_entity);

        selected_tiles.select_single(new_entity);
        assert_eq!(selected_tiles.selection().len(), 1);
        assert!(!selected_tiles.contains_tile(existing_entity));
        assert!(selected_tiles.contains_tile(new_entity));
    }

    #[test]
    fn select_single_already_selected() {
        let mut selected_tiles = SelectedTiles::default();
        let existing_entity = Entity::from_bits(0);

        selected_tiles.add_tile(existing_entity);

        selected_tiles.select_single(existing_entity);
        assert_eq!(selected_tiles.selection().len(), 0);
        assert!(!selected_tiles.contains_tile(existing_entity));
    }
}
