//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use hexx::HexIterExt;
use leafwing_input_manager::prelude::ActionState;

use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

use crate as emergence_lib;

use super::clipboard::Clipboard;
use super::{cursor::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSelection>()
            .init_resource::<SelectionState>()
            .init_resource::<HoveredTiles>()
            .add_system(
                set_selection
                    .in_set(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            )
            .add_system(
                set_tile_interactions
                    .in_set(InteractionSystem::SelectTiles)
                    .after(set_selection),
            )
            .add_system(update_selection_radius);
    }
}

/// The set of tiles that is currently selected
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SelectedTiles {
    /// Actively selected tiles
    selected: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    pub(super) fn add_tile(&mut self, tile_pos: TilePos) {
        self.selected.insert(tile_pos);
    }

    /// Deselects a single tile
    #[cfg(test)]
    fn remove_tile(&mut self, tile_pos: TilePos) {
        self.selected.remove(&tile_pos);
    }

    /// Is the given tile in the selection?
    pub(crate) fn contains_tile(&self, tile_pos: TilePos) -> bool {
        self.selected.contains(&tile_pos)
    }

    /// Computes the center of the selection
    pub(crate) fn center(&self) -> TilePos {
        TilePos {
            hex: self.selected.iter().map(|tile_pos| tile_pos.hex).center(),
        }
    }

    /// Draws a hollow hexagonal ring of tiles.
    fn draw_ring(center: TilePos, radius: u32) -> HashSet<TilePos> {
        let hex_coord = center.ring(radius);
        HashSet::from_iter(hex_coord.into_iter().map(|hex| TilePos { hex }))
    }

    /// Draws a hexagon of tiles.
    fn draw_hexagon(center: TilePos, radius: u32) -> HashSet<TilePos> {
        let hex_coord = hexagon(center.hex, radius);
        HashSet::from_iter(hex_coord.map(|hex| TilePos { hex }))
    }

    /// Computes the set of hexagons between `start` and `end`, with a thickness determnind by `radius`.
    fn draw_line(start: TilePos, end: TilePos, radius: u32) -> HashSet<TilePos> {
        let line = start.line_to(end.hex);
        let mut tiles = HashSet::<TilePos>::new();

        for line_hex in line {
            let hexagon = hexagon(line_hex, radius);
            for hex in hexagon {
                tiles.insert(TilePos { hex });
            }
        }
        tiles
    }

    /// Clears the set of selected tiles.
    pub(super) fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// The set of currently selected tiles.
    pub(crate) fn selection(&self) -> &HashSet<TilePos> {
        &self.selected
    }

    /// Are any tiles selected?
    pub(super) fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Handles all of the logic needed to add tiles to the selection.
    fn add_to_selection(
        &mut self,
        hovered_tile: TilePos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        let selection_region =
            self.compute_selection_region(hovered_tile, selection_state, map_geometry);

        self.selected = match selection_state.multiple {
            true => HashSet::from_iter(self.selected.union(&selection_region).copied()),
            false => selection_region,
        }
    }

    /// Handles all of the logic needed to remove tiles from the selection.
    fn remove_from_selection(
        &mut self,
        hovered_tile: TilePos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        if selection_state.multiple {
            let selection_region =
                self.compute_selection_region(hovered_tile, selection_state, map_geometry);

            self.selected =
                HashSet::from_iter(self.selected.difference(&selection_region).copied());
        } else {
            self.clear_selection()
        }
    }

    /// Returns the set of tiles that should be modified by any selection action.
    fn compute_selection_region(
        &self,
        hovered_tile: TilePos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) -> HashSet<TilePos> {
        match selection_state.shape {
            SelectionShape::Single => {
                SelectedTiles::draw_hexagon(hovered_tile, selection_state.brush_size)
            }
            SelectionShape::Area { center, radius } => SelectedTiles::draw_hexagon(center, radius),
            SelectionShape::Line { start } => {
                SelectedTiles::draw_line(start, hovered_tile, selection_state.brush_size)
            }
        }
        // PERF: we could be faster about this by only collecting once
        .into_iter()
        // Ensure we don't try to operate off of the map
        .filter(|tile_pos| map_geometry.is_valid(*tile_pos))
        .collect()
    }

    /// Fetches the entities that correspond to these tiles
    pub(crate) fn entities(&self, map_geometry: &MapGeometry) -> Vec<Entity> {
        self.selection()
            .iter()
            .flat_map(|tile_pos| map_geometry.get_terrain(*tile_pos))
            .collect()
    }
}

/// The set of tiles that are being hovered
#[derive(Resource, Debug, Default, Deref)]
pub(crate) struct HoveredTiles {
    /// The set of tiles that are hovered over
    hovered: HashSet<TilePos>,
}

impl HoveredTiles {
    /// Updates the set of hovered actions based on the current cursor position and player inputs.
    fn update(&mut self, hovered_tile: TilePos, selection_state: &SelectionState) {
        self.hovered = match selection_state.shape {
            SelectionShape::Single => {
                SelectedTiles::draw_hexagon(hovered_tile, selection_state.brush_size)
            }
            SelectionShape::Area { center, radius } => {
                let mut set = SelectedTiles::draw_ring(center, radius);
                // Also show center of ring for clarity.
                set.insert(hovered_tile);
                set
            }
            SelectionShape::Line { start } => {
                SelectedTiles::draw_line(start, hovered_tile, selection_state.brush_size)
            }
        };
    }
}

/// How a given object is being interacted with by the player.
#[derive(Component, PartialEq, Eq, Hash, Clone, Debug, IterableEnum, Default)]
pub(crate) enum ObjectInteraction {
    /// Currently in the selection.
    Selected,
    /// Hovered over with the cursor.
    Hovered,
    /// Hovered over and simultaneously selected.
    ///
    /// This exists to allow easy visual distinction of this state,
    /// and should include visual elements of both.
    ///
    // TODO: this is silly and probably shouldn't exist, but we're using colors for everything for now so...
    // Tracked in https://github.com/Leafwing-Studios/Emergence/issues/263
    HoveredAndSelected,
    /// Not in the object or the selection
    #[default]
    None,
}

impl ObjectInteraction {
    /// Constructs a new [`ObjectInteraction`]
    pub(crate) fn new(hovered: bool, selected: bool) -> Self {
        match (hovered, selected) {
            (true, true) => ObjectInteraction::HoveredAndSelected,
            (true, false) => ObjectInteraction::Hovered,
            (false, true) => ObjectInteraction::Selected,
            (false, false) => ObjectInteraction::None,
        }
    }

    /// The material used by objects that are being interacted with.
    pub(crate) fn material(&self) -> Option<StandardMaterial> {
        use crate::asset_management::palette::infovis::{
            HOVER_COLOR, SELECTION_AND_HOVER_COLOR, SELECTION_COLOR,
        };

        let maybe_color = match self {
            ObjectInteraction::Selected => Some(SELECTION_COLOR),
            ObjectInteraction::Hovered => Some(HOVER_COLOR),
            ObjectInteraction::HoveredAndSelected => Some(SELECTION_AND_HOVER_COLOR),
            ObjectInteraction::None => None,
        };

        // Prevent z-fighting between ghosts and previews
        let depth_bias = match self {
            ObjectInteraction::Selected => 1.,
            ObjectInteraction::Hovered => 2.,
            ObjectInteraction::HoveredAndSelected => 3.,
            ObjectInteraction::None => 4.,
        };

        maybe_color.map(|base_color| StandardMaterial {
            base_color,
            alpha_mode: AlphaMode::Blend,
            depth_bias,
            ..Default::default()
        })
    }
}

/// Sets the radius of "brush" used to select tiles.
fn update_selection_radius(
    mut selection_state: ResMut<SelectionState>,
    actions: Res<ActionState<PlayerAction>>,
) {
    if actions.just_pressed(PlayerAction::IncreaseSelectionRadius) {
        // This max brush size is set
        selection_state.brush_size = (selection_state.brush_size + 1).min(10);
    }

    if actions.just_pressed(PlayerAction::DecreaseSelectionRadius) {
        selection_state.brush_size = selection_state.brush_size.saturating_sub(1);
    }
}

/// The game object(s) currently selected for inspection.
#[derive(Resource, Debug, Default)]
pub(crate) enum CurrentSelection {
    /// A ghost is selected
    Ghost(Entity),
    /// A structure is selected
    Structure(Entity),
    /// One or more tile is selected
    Terrain(SelectedTiles),
    /// A unit is selected
    Unit(Entity),
    /// Nothing is selected
    #[default]
    None,
}

impl CurrentSelection {
    /// Returns the set of terrain tiles that should be affected by actions.
    pub(super) fn relevant_tiles(&self, cursor_pos: &CursorPos) -> SelectedTiles {
        match self {
            CurrentSelection::Terrain(selected_tiles) => match selected_tiles.is_empty() {
                true => {
                    let mut selected_tiles = SelectedTiles::default();
                    if let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() {
                        selected_tiles.add_tile(cursor_tile_pos);
                    }
                    selected_tiles
                }
                false => selected_tiles.clone(),
            },
            _ => {
                let mut selected_tiles = SelectedTiles::default();
                if let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() {
                    selected_tiles.add_tile(cursor_tile_pos);
                }
                selected_tiles
            }
        }
    }

    /// Just select the terrain.
    #[must_use]
    fn select_terrain(
        &self,
        hovered_tile: TilePos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) -> Self {
        if let CurrentSelection::Terrain(existing_selection) = self {
            let mut existing_selection = existing_selection.clone();
            existing_selection.add_to_selection(hovered_tile, selection_state, map_geometry);
            CurrentSelection::Terrain(existing_selection)
        } else {
            let mut selected_tiles = SelectedTiles::default();
            selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
            CurrentSelection::Terrain(selected_tiles)
        }
    }

    /// Determines the selection based on the cursor information.
    ///
    /// This handles the simple case, when we're selecting a new tile.
    /// Ordinarily, just prioritize units > structures > terrain
    fn update_from_cursor_pos(
        &mut self,
        cursor_pos: &CursorPos,
        hovered_tile: TilePos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        *self = if selection_state.multiple {
            self.select_terrain(hovered_tile, selection_state, map_geometry)
        } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
            CurrentSelection::Unit(unit_entity)
        } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
            CurrentSelection::Structure(structure_entity)
        } else {
            self.select_terrain(hovered_tile, selection_state, map_geometry)
        }
    }

    /// Cycles through game objects on the same tile.
    ///
    /// The order is units -> ghosts -> structures -> terrain -> units.
    /// If a higher priority option is missing, later options in the chain are searched.
    /// If none of the options can be found, the selection is cleared completely.
    fn cycle_selection(
        &mut self,
        cursor_pos: &CursorPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        *self = match self {
            CurrentSelection::None => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(ghost_entity) = cursor_pos.maybe_ghost() {
                    CurrentSelection::Ghost(ghost_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
                    CurrentSelection::Terrain(selected_tiles)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Ghost(_) => {
                if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
                    CurrentSelection::Terrain(selected_tiles)
                } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(ghost_entity) = cursor_pos.maybe_ghost() {
                    CurrentSelection::Ghost(ghost_entity)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Structure(_) => {
                if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
                    CurrentSelection::Terrain(selected_tiles)
                } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(ghost_entity) = cursor_pos.maybe_ghost() {
                    CurrentSelection::Ghost(ghost_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Terrain(existing_selection) => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(ghost_entity) = cursor_pos.maybe_ghost() {
                    CurrentSelection::Ghost(ghost_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    existing_selection.add_to_selection(
                        hovered_tile,
                        selection_state,
                        map_geometry,
                    );
                    CurrentSelection::Terrain(existing_selection.clone())
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Unit(_) => {
                if let Some(ghost_entity) = cursor_pos.maybe_ghost() {
                    CurrentSelection::Ghost(ghost_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
                    CurrentSelection::Terrain(selected_tiles)
                } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else {
                    CurrentSelection::None
                }
            }
        }
    }
}

/// Tracks what should be done with the selection (and hovered tiles) this frame.
#[derive(Resource, Default, Debug)]
struct SelectionState {
    /// What is the shape of the selection?
    shape: SelectionShape,
    /// What should be done to the selection?
    action: SelectionAction,
    /// Should the selection be erased or modified?
    multiple: bool,
    /// The selection size to use for non-Area selections
    brush_size: u32,
}

/// What should be done with the selected tiles
#[derive(Default, Debug, Clone, Copy)]
enum SelectionAction {
    /// Just highlight them
    #[default]
    Preview,
    /// Add them to the select
    Select,
    /// Remove them from the selection
    Deselect,
}

/// The shape of tiles to be selected.
#[derive(Default, Debug, Clone, Copy)]
enum SelectionShape {
    /// A single tile (or a large brush equivalent)
    #[default]
    Single,
    /// A regular hexagon
    Area {
        /// The center of the hexagon
        center: TilePos,
        /// The distance to each corner of the hexagon, in tiles
        radius: u32,
    },
    /// A discretized line
    Line {
        /// The start of the line
        start: TilePos,
    },
}

impl SelectionState {
    /// Determine what selection state should be used this frame based on player actions
    fn compute(
        &mut self,
        clipboard: &Clipboard,
        actions: &ActionState<PlayerAction>,
        hovered_tile: TilePos,
    ) {
        use PlayerAction::*;

        self.multiple = actions.pressed(PlayerAction::Multiple);

        self.shape = if actions.pressed(Line) {
            let start = if let SelectionShape::Line { start } = self.shape {
                start
            } else {
                hovered_tile
            };

            SelectionShape::Line { start }
        } else if actions.pressed(Area) {
            let center = if let SelectionShape::Area { center, .. } = self.shape {
                center
            } else {
                hovered_tile
            };
            let radius = hovered_tile.unsigned_distance_to(center.hex);

            SelectionShape::Area { center, radius }
        } else {
            SelectionShape::Single
        };

        self.action = match self.shape {
            SelectionShape::Single => {
                if actions.pressed(Select) {
                    SelectionAction::Select
                // Don't repeatedly trigger deselect to avoid accidentally clearing selection
                } else if actions.just_pressed(Deselect) {
                    SelectionAction::Deselect
                } else {
                    SelectionAction::Preview
                }
            }
            SelectionShape::Area { .. } | SelectionShape::Line { .. } => {
                // Trigger on just released in order to enable a drag-and-preview effect
                if actions.just_released(Select) {
                    SelectionAction::Select
                } else if actions.just_released(Deselect) {
                    SelectionAction::Deselect
                } else {
                    SelectionAction::Preview
                }
            }
        };

        // If the clipboard is not empty, PlayerAction::Select is used to paste from the clipboard instead of selecting.
        // Allow users to override this using shift+select to expand or shrink their selection.
        if !clipboard.is_empty() && !self.multiple {
            self.action = SelectionAction::Preview;
        }
    }
}

/// Determine what should be selected based on player inputs.
fn set_selection(
    clipboard: Res<Clipboard>,
    mut current_selection: ResMut<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    mut hovered_tiles: ResMut<HoveredTiles>,
    mut selection_state: ResMut<SelectionState>,
    mut last_tile_selected: Local<Option<TilePos>>,
    map_geometry: Res<MapGeometry>,
) {
    // Cast to ordinary references for ease of use
    let actions = &*actions;
    let cursor_pos = &*cursor_pos;
    let map_geometry = &*map_geometry;

    let Some(hovered_tile) = cursor_pos.maybe_tile_pos() else {return};

    // Compute how we should handle the selection based on the actions of the player
    selection_state.compute(&clipboard, actions, hovered_tile);

    // Update hovered tiles
    hovered_tiles.update(hovered_tile, &selection_state);

    // Select and deselect tiles
    match (selection_state.action, selection_state.shape) {
        // No need to do work here, hovered tiles are always computed
        (SelectionAction::Preview, _) => (),
        (SelectionAction::Select, SelectionShape::Line { .. }) => {
            *current_selection =
                current_selection.select_terrain(hovered_tile, &selection_state, map_geometry);
            // Let players chain lines head to tail nicely
            selection_state.shape = SelectionShape::Line {
                start: hovered_tile,
            };
        }
        (SelectionAction::Select, SelectionShape::Area { .. }) => {
            *current_selection =
                current_selection.select_terrain(hovered_tile, &selection_state, map_geometry);
        }
        (SelectionAction::Select, SelectionShape::Single) => {
            // If we can compare them, do
            let same_tile_as_last_time = if let (Some(last_pos), Some(current_pos)) =
                (*last_tile_selected, cursor_pos.maybe_tile_pos())
            {
                last_pos == current_pos
            } else {
                false
            };
            // Update the cache
            *last_tile_selected = cursor_pos.maybe_tile_pos();

            if same_tile_as_last_time
                && !selection_state.multiple
                && actions.just_pressed(PlayerAction::Select)
            {
                current_selection.cycle_selection(cursor_pos, &selection_state, map_geometry)
            } else if !same_tile_as_last_time {
                current_selection.update_from_cursor_pos(
                    cursor_pos,
                    hovered_tile,
                    &selection_state,
                    map_geometry,
                )
            }
        }
        (SelectionAction::Deselect, SelectionShape::Area { .. } | SelectionShape::Single) => {
            match &mut *current_selection {
                CurrentSelection::Terrain(ref mut selected_tiles) => {
                    if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                        selected_tiles.remove_from_selection(
                            hovered_tile,
                            &selection_state,
                            map_geometry,
                        );
                    }
                }
                _ => *current_selection = CurrentSelection::None,
            }
        }
        (SelectionAction::Deselect, SelectionShape::Line { .. }) => {
            match &mut *current_selection {
                CurrentSelection::Terrain(ref mut selected_tiles) => {
                    if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                        selected_tiles.remove_from_selection(
                            hovered_tile,
                            &selection_state,
                            map_geometry,
                        );
                    }
                }
                _ => *current_selection = CurrentSelection::None,
            }

            // Let players chain lines head to tail nicely
            selection_state.shape = SelectionShape::Line {
                start: hovered_tile,
            };
        }
    }
}

/// Set tile interactions based on hover and selection state
pub(super) fn set_tile_interactions(
    current_selection: Res<CurrentSelection>,
    hovered_tiles: Res<HoveredTiles>,
    mut terrain_query: Query<(&TilePos, &mut ObjectInteraction)>,
) {
    if current_selection.is_changed() || hovered_tiles.is_changed() {
        for (&tile_pos, mut object_interaction) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&tile_pos);
            let selected = if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
                selected_tiles.contains_tile(tile_pos)
            } else {
                false
            };

            *object_interaction = ObjectInteraction::new(hovered, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedTiles;
    use crate::{
        player_interaction::{cursor::CursorPos, selection::CurrentSelection},
        simulation::geometry::TilePos,
    };

    #[test]
    fn simple_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(tile_pos);
        assert!(selected_tiles.contains_tile(tile_pos));
        assert!(!selected_tiles.is_empty());
        assert_eq!(selected_tiles.selected.len(), 1);

        selected_tiles.remove_tile(tile_pos);
        assert!(!selected_tiles.contains_tile(tile_pos));
        assert!(selected_tiles.is_empty());
        assert_eq!(selected_tiles.selected.len(), 0);
    }

    #[test]
    fn multi_select() {
        let mut selected_tiles = SelectedTiles::default();

        selected_tiles.add_tile(TilePos::new(1, 1));
        // Intentionally doubled
        selected_tiles.add_tile(TilePos::new(1, 1));
        selected_tiles.add_tile(TilePos::new(2, 2));
        selected_tiles.add_tile(TilePos::new(3, 3));

        assert_eq!(selected_tiles.selected.len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_tiles = SelectedTiles::default();
        selected_tiles.add_tile(TilePos::new(1, 1));
        selected_tiles.add_tile(TilePos::new(2, 2));
        selected_tiles.add_tile(TilePos::new(3, 3));

        assert_eq!(selected_tiles.selected.len(), 3);
        selected_tiles.clear_selection();
        assert_eq!(selected_tiles.selected.len(), 0);
    }

    #[test]
    fn relevant_tiles_returns_cursor_pos_with_empty_selection() {
        let cursor_pos = CursorPos::new(TilePos::new(24, 7));
        let mut cursor_pos_selected = SelectedTiles::default();
        cursor_pos_selected.add_tile(cursor_pos.maybe_tile_pos().unwrap());

        assert_eq!(
            CurrentSelection::None.relevant_tiles(&cursor_pos),
            cursor_pos_selected
        );

        assert_eq!(
            CurrentSelection::Terrain(SelectedTiles::default()).relevant_tiles(&cursor_pos),
            cursor_pos_selected
        );
    }
}
