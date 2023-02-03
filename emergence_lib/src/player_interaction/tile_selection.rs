//! Selecting tiles to be built on, inspected or modified

use bevy::{prelude::*, utils::HashSet};
use hexx::shapes::hexagon;
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::{InputKind, Modifier, UserInput},
    Actionlike,
};
use petitset::PetitSet;

use crate::{
    asset_management::TileHandles,
    simulation::geometry::{MapGeometry, TilePos},
    terrain::Terrain,
};

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
    /// Selects or deselects a group of hex tiles by dragging over them
    ///
    /// This action will track whether you are selecting or deselecting tiles based on the state of the first tile modified with this action.
    Multiple,
    /// Selects or deselects a broad swath of hex tiles by dragging over them
    ///
    /// This action will track whether you are selecting or deselecting tiles based on the state of the first tile modified with this action.
    AreaMultiple,
    /// Selects a large hexagon around the cursor, based on the second position clicked.
    Hexagonal,
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
        let mut control_shift_left_click = PetitSet::<InputKind, 8>::new();
        control_shift_left_click.insert(Modifier::Control.into());
        control_shift_left_click.insert(Modifier::Shift.into());
        control_shift_left_click.insert(MouseButton::Left.into());

        InputMap::new([
            (
                UserInput::Single(InputKind::Mouse(MouseButton::Left)),
                TileSelectionAction::Single,
            ),
            (
                UserInput::modified(Modifier::Shift, MouseButton::Left),
                TileSelectionAction::Multiple,
            ),
            (
                UserInput::Chord(control_shift_left_click),
                TileSelectionAction::AreaMultiple,
            ),
            (
                UserInput::modified(Modifier::Control, MouseButton::Left),
                TileSelectionAction::Hexagonal,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Escape)),
                TileSelectionAction::Clear,
            ),
        ])
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default, Clone)]
pub struct SelectedTiles {
    /// Actively selected tiles
    selection: HashSet<(Entity, TilePos)>,
}

impl SelectedTiles {
    /// Selects a single tile
    pub fn add_tile(&mut self, tile_entity: Entity, tile_pos: TilePos) {
        self.selection.insert((tile_entity, tile_pos));
    }

    /// Deselects a single tile
    pub fn remove_tile(&mut self, tile_entity: Entity, tile_pos: TilePos) {
        self.selection.remove(&(tile_entity, tile_pos));
    }

    /// Selects a single tile, at the expense of any other tiles already selected.
    ///
    /// If a tile is not selected, select it.
    /// If a tile is already selected, remove it from the selection.
    ///
    /// This is the behavior controlled by [`TileSelectionAction::Single`].
    pub fn select_single(&mut self, tile_entity: Entity, tile_pos: TilePos) {
        if self.selection.contains(&(tile_entity, tile_pos)) {
            self.selection.clear();
        } else {
            // Clear cache then reinsert in the previous cache structure rather than making a new one
            // to avoid a pointless reallocation
            self.selection.clear();
            self.selection.insert((tile_entity, tile_pos));
        }
    }

    /// Selects a hexagon of tiles.
    fn select_hexagon(
        &mut self,
        center: TilePos,
        radius: u32,
        map_geometry: &MapGeometry,
        selection_mode: &SelectMode,
    ) {
        let hex_coord = hexagon(center.hex, radius);

        for hex in hex_coord {
            let target_pos = TilePos { hex };
            // Selection may have overflowed map
            if let Some(target_entity) = map_geometry.terrain_index.get(&target_pos) {
                match *selection_mode {
                    SelectMode::Select => self.add_tile(*target_entity, target_pos),
                    SelectMode::Deselect => self.remove_tile(*target_entity, target_pos),
                    SelectMode::None => unreachable!(),
                }
            }
        }
    }

    /// The current set of selected tiles
    pub fn selection(&self) -> &HashSet<(Entity, TilePos)> {
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
    pub fn contains_tile(&self, tile_entity: Entity, tile_pos: TilePos) -> bool {
        self.selection.contains(&(tile_entity, tile_pos))
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
#[allow(clippy::too_many_arguments)]
fn select_tiles(
    cursor: Res<CursorPos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<TileSelectionAction>>,
    mut selection_mode: Local<SelectMode>,
    mut selection_start: Local<Option<TilePos>>,
    mut initial_selection: Local<Option<SelectedTiles>>,
    mut previous_radius: Local<u32>,
    map_geometry: Res<MapGeometry>,
) {
    if let (Some(cursor_entity), Some(cursor_tile)) =
        (cursor.maybe_entity(), cursor.maybe_tile_pos())
    {
        if actions.pressed(TileSelectionAction::Clear) {
            selected_tiles.clear_selection();
        };

        if *selection_mode == SelectMode::None {
            *selection_mode = match selected_tiles.contains_tile(cursor_entity, cursor_tile) {
                // If you start with a selected tile, subtract from the selection
                true => SelectMode::Deselect,
                // If you start with an unselected tile, add to the selection
                false => SelectMode::Select,
            }
        }

        if actions.pressed(TileSelectionAction::AreaMultiple) {
            selected_tiles.select_hexagon(
                cursor_tile,
                *previous_radius,
                map_geometry.as_ref(),
                &selection_mode,
            );
        } else if actions.pressed(TileSelectionAction::Multiple) {
            match *selection_mode {
                SelectMode::Select => selected_tiles.add_tile(cursor_entity, cursor_tile),
                SelectMode::Deselect => selected_tiles.remove_tile(cursor_entity, cursor_tile),
                SelectMode::None => unreachable!(),
            }
        } else if actions.pressed(TileSelectionAction::Hexagonal) {
            if selection_start.is_none() {
                *selection_start = Some(cursor_tile);
                *initial_selection = Some(selected_tiles.clone());
            }

            let radius = cursor_tile.unsigned_distance_to(selection_start.unwrap().hex);
            *previous_radius = radius;

            // We need to be able to expand and shrink the selection reversibly
            // so we need a snapshot of the state before this action took place.
            *selected_tiles = initial_selection.as_ref().unwrap().clone();
            selected_tiles.select_hexagon(
                selection_start.unwrap(),
                radius,
                map_geometry.as_ref(),
                &selection_mode,
            )
        } else {
            *selection_mode = SelectMode::None;
        };

        if actions.released(TileSelectionAction::Hexagonal) {
            *selection_start = None;
            *initial_selection = None;
        }

        if actions.just_pressed(TileSelectionAction::Single) {
            selected_tiles.select_single(cursor_entity, cursor_tile);
        }
    }
}

/// Highlights the current set of selected tiles
fn highlight_selected_tiles(
    selected_tiles: Res<SelectedTiles>,
    mut terrain_query: Query<(Entity, &mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TileHandles>,
) {
    if selected_tiles.is_changed() {
        let selection = selected_tiles.selection();
        // PERF: We should probably avoid a linear scan over all tiles here
        for (terrain_entity, mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            if selection.contains(&(terrain_entity, tile_pos)) {
                *material = materials.selected_tile_handle.clone_weak();
            } else {
                // FIXME: reset to the correct material
                *material = materials.terrain_handles.get(terrain).unwrap().clone_weak();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedTiles;
    use crate::simulation::geometry::TilePos;
    use bevy::ecs::entity::Entity;

    #[test]
    fn simple_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_entity = Entity::from_bits(0);
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(tile_entity, tile_pos);
        assert!(selected_tiles.contains_tile(tile_entity, tile_pos));
        assert!(!selected_tiles.is_empty());
        assert_eq!(selected_tiles.selection().len(), 1);

        selected_tiles.remove_tile(tile_entity, tile_pos);
        assert!(!selected_tiles.contains_tile(tile_entity, tile_pos));
        assert!(selected_tiles.is_empty());
        assert_eq!(selected_tiles.selection().len(), 0);
    }

    #[test]
    fn multi_select() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(Entity::from_bits(0), tile_pos);
        // Intentionally doubled
        selected_tiles.add_tile(Entity::from_bits(0), tile_pos);
        selected_tiles.add_tile(Entity::from_bits(1), tile_pos);
        selected_tiles.add_tile(Entity::from_bits(2), tile_pos);

        assert_eq!(selected_tiles.selection().len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(Entity::from_bits(0), tile_pos);
        selected_tiles.add_tile(Entity::from_bits(1), tile_pos);
        selected_tiles.add_tile(Entity::from_bits(2), tile_pos);

        assert_eq!(selected_tiles.selection().len(), 3);
        selected_tiles.clear_selection();
        assert_eq!(selected_tiles.selection().len(), 0);
    }

    #[test]
    fn select_single_not_yet_selected() {
        let mut selected_tiles = SelectedTiles::default();
        let existing_entity = Entity::from_bits(0);
        let new_entity = Entity::from_bits(1);
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(existing_entity, tile_pos);

        selected_tiles.select_single(new_entity, tile_pos);
        assert_eq!(selected_tiles.selection().len(), 1);
        assert!(!selected_tiles.contains_tile(existing_entity, tile_pos));
        assert!(selected_tiles.contains_tile(new_entity, tile_pos));
    }

    #[test]
    fn select_single_already_selected() {
        let mut selected_tiles = SelectedTiles::default();
        let existing_entity = Entity::from_bits(0);
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(existing_entity, tile_pos);

        selected_tiles.select_single(existing_entity, tile_pos);
        assert_eq!(selected_tiles.selection().len(), 0);
        assert!(!selected_tiles.contains_tile(existing_entity, tile_pos));
    }
}
