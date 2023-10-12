//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use hexx::Hex;
use hexx::HexIterExt;
use leafwing_input_manager::prelude::ActionState;

use crate::geometry::MapGeometry;
use crate::geometry::VoxelObject;
use crate::geometry::VoxelPos;

use crate as emergence_lib;

use super::clipboard::Tool;
use super::{picking::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSelection>()
            .init_resource::<SelectionState>()
            .init_resource::<HoveredTiles>()
            .add_systems(
                Update,
                set_selection
                    .in_set(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            )
            .add_systems(
                Update,
                set_tile_interactions
                    .in_set(InteractionSystem::SelectTiles)
                    .after(set_selection),
            )
            .add_systems(Update, update_selection_radius);
    }
}

/// The set of voxels that are currently selected
#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub(crate) struct SelectedVoxels {
    /// The underlying set of voxels
    selected: HashSet<VoxelPos>,
}

impl SelectedVoxels {
    /// Gets the entities stored in the selected set of voxel positions.
    pub(crate) fn voxel_objects(&self, map_geometry: &MapGeometry) -> HashSet<VoxelObject> {
        self.selected
            .iter()
            .flat_map(|voxel| map_geometry.get_voxel(*voxel))
            .cloned()
            .collect()
    }

    /// Computes the center of the selection
    pub(crate) fn center(&self) -> Hex {
        self.selected.iter().map(|voxel| voxel.hex).center()
    }

    /// Draws a hollow hexagonal ring of tiles.
    fn draw_ring(center: VoxelPos, radius: u32) -> Vec<Hex> {
        center.hex.ring(radius).collect()
    }

    /// Draws a hexagon of tiles.
    fn draw_hexagon(center: VoxelPos, radius: u32) -> Vec<Hex> {
        hexagon(center.hex, radius).collect()
    }

    /// Computes the set of hexagons between `start` and `end`, with a thickness determnind by `radius`.
    fn draw_line(start: VoxelPos, end: VoxelPos, radius: u32) -> Vec<Hex> {
        if radius == 0 {
            start.hex.line_to(end.hex).collect()
        } else {
            let mut hex_vec = Vec::new();
            for central_hex in start.hex.line_to(end.hex) {
                hex_vec.extend(hexagon(central_hex, radius));
            }
            hex_vec
        }
    }

    /// Handles all of the logic needed to add tiles to the selection.
    fn add_to_selection(
        &mut self,
        hovered_tile: VoxelPos,
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
        hovered_tile: VoxelPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        if selection_state.multiple {
            let selection_region =
                self.compute_selection_region(hovered_tile, selection_state, map_geometry);

            self.selected =
                HashSet::from_iter(self.selected.difference(&selection_region).copied());
        } else {
            self.clear()
        }
    }

    /// Returns the set of tiles that should be modified by any selection action.
    fn compute_selection_region(
        &self,
        hovered_tile: VoxelPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) -> HashSet<VoxelPos> {
        match selection_state.shape {
            SelectionShape::Single => HashSet::from_iter([hovered_tile]),
            SelectionShape::Area { center, radius } => SelectedVoxels::draw_hexagon(center, radius)
                .iter()
                .filter(|hex| map_geometry.is_valid(**hex))
                .map(|hex| VoxelPos {
                    hex: *hex,
                    height: map_geometry.get_height(*hex).unwrap(),
                })
                .collect(),
            SelectionShape::Line { start } => {
                SelectedVoxels::draw_line(start, hovered_tile, selection_state.brush_size)
                    .iter()
                    .filter(|hex| map_geometry.is_valid(**hex))
                    .map(|hex| VoxelPos {
                        hex: *hex,
                        height: map_geometry.get_height(*hex).unwrap(),
                    })
                    .collect()
            }
        }
    }
}

/// The set of tiles that are being hovered
#[derive(Resource, Debug, Default, Deref)]
pub(crate) struct HoveredTiles {
    /// The set of tiles that are hovered over
    // FIXME: these should probably store a VoxelPos instead of a Hex
    hovered: HashSet<Hex>,
}

impl HoveredTiles {
    /// Updates the set of hovered actions based on the current cursor position and player inputs.
    fn update(&mut self, hovered_tile: VoxelPos, selection_state: &SelectionState) {
        let hex_vec = match selection_state.shape {
            SelectionShape::Single => {
                SelectedVoxels::draw_hexagon(hovered_tile, selection_state.brush_size)
            }
            SelectionShape::Area { center, radius } => {
                let mut vec = SelectedVoxels::draw_ring(center, radius);
                // Also show center of ring for clarity.
                vec.push(hovered_tile.hex);
                vec
            }
            SelectionShape::Line { start } => {
                SelectedVoxels::draw_line(start, hovered_tile, selection_state.brush_size)
            }
        };

        self.hovered = HashSet::from_iter(hex_vec);
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
        use crate::graphics::palette::infovis::{
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
    /// One or more tile is selected.
    ///
    /// Note that terraforming details are also displayed on the basis of the selected terrain.
    Voxels(SelectedVoxels),
    /// A unit is selected
    Unit(Entity),
    /// Nothing is selected
    #[default]
    None,
}

impl CurrentSelection {
    /// Returns the set of terrain tiles that should be affected by actions.
    pub(crate) fn relevant_tiles(&self, cursor_pos: &CursorPos) -> SelectedVoxels {
        match self {
            CurrentSelection::Voxels(selected_voxels) => match selected_voxels.is_empty() {
                true => {
                    let mut selected_voxels = SelectedVoxels::default();
                    if let Some(cursor_tile_pos) = cursor_pos.maybe_voxel_pos() {
                        selected_voxels.insert(cursor_tile_pos);
                    }
                    selected_voxels
                }
                false => selected_voxels.clone(),
            },
            _ => {
                let mut selected_voxels = SelectedVoxels::default();
                if let Some(cursor_tile_pos) = cursor_pos.maybe_voxel_pos() {
                    selected_voxels.insert(cursor_tile_pos);
                }
                selected_voxels
            }
        }
    }

    /// Just select the terrain.
    #[must_use]
    fn select_terrain(
        &self,
        hovered_tile: VoxelPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) -> Self {
        if let CurrentSelection::Voxels(existing_selection) = self {
            let mut existing_selection = existing_selection.clone();
            existing_selection.add_to_selection(hovered_tile, selection_state, map_geometry);
            CurrentSelection::Voxels(existing_selection)
        } else {
            let mut selected_voxels = SelectedVoxels::default();
            selected_voxels.add_to_selection(hovered_tile, selection_state, map_geometry);
            CurrentSelection::Voxels(selected_voxels)
        }
    }

    /// Determines the selection based on the cursor information.
    ///
    /// This handles the simple case, when we're selecting a new tile.
    /// Ordinarily, just prioritize units > structures > terrain
    fn update_from_cursor_pos(
        &mut self,
        cursor_pos: &CursorPos,
        hovered_tile: VoxelPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        *self = if selection_state.multiple {
            self.select_terrain(hovered_tile, selection_state, map_geometry)
        } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
            CurrentSelection::Unit(unit_entity)
        } else {
            self.select_terrain(hovered_tile, selection_state, map_geometry)
        }
    }

    /// Cycles through game objects on the same tile.
    ///
    /// The order is units -> ghost structure -> ghost terrain -> structures -> terrain -> units.
    /// If a higher priority option is missing, later options in the chain are searched.
    /// If none of the options can be found, the selection is cleared completely.
    fn cycle_selection(
        &mut self,
        cursor_pos: &CursorPos,
        selection_state: &SelectionState,
        map_geometry: &MapGeometry,
    ) {
        let start = SelectionVariant::from(&*self);
        let cycle = start.cycle();

        for variant in cycle {
            if let Some(selection) = self.get(variant, selection_state, cursor_pos, map_geometry) {
                *self = selection;
                return;
            }
        }

        *self = CurrentSelection::None;
    }

    /// Try to select an object of a type corresponding to the `selection_variant`.
    ///
    /// If it cannot be found, [`None`] is returned.
    fn get(
        &self,
        selection_variant: SelectionVariant,
        selection_state: &SelectionState,
        cursor_pos: &CursorPos,
        map_geometry: &MapGeometry,
    ) -> Option<Self> {
        match selection_variant {
            SelectionVariant::Unit => cursor_pos.maybe_unit().map(CurrentSelection::Unit),
            SelectionVariant::Voxel => {
                let hovered_tile = cursor_pos.maybe_voxel_pos()?;
                let mut selected_voxels = SelectedVoxels::default();
                selected_voxels.add_to_selection(hovered_tile, selection_state, map_geometry);
                Some(CurrentSelection::Voxels(selected_voxels))
            }
            SelectionVariant::None => None,
        }
    }
}

/// The dataless enum that tracks the variety of [`CurrentSelection`].
#[derive(IterableEnum, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum SelectionVariant {
    /// A unit.
    Unit,
    /// A voxel
    Voxel,
    /// No selection.
    None,
}

impl SelectionVariant {
    /// Get the next selection mode in the chain.
    ///
    /// The order is units -> ghost structure -> ghost terrain -> structures -> terrain -> units.
    /// No path leads to None: it is instead the fallback if nothing can be found.
    fn next(&self) -> Self {
        match self {
            Self::None => Self::Unit,
            Self::Unit => Self::Voxel,
            Self::Voxel => Self::Unit,
        }
    }

    /// Returns the cycle order for the given `start`.
    fn cycle(&self) -> Vec<Self> {
        if self == &Self::None {
            return vec![];
        }

        let mut cycle = Vec::new();
        let mut next = self.next();
        while next != *self {
            cycle.push(next);
            next = next.next();
        }

        // Fallback to self if nothing else is found
        cycle.push(*self);

        cycle
    }
}

impl From<&CurrentSelection> for SelectionVariant {
    fn from(selection: &CurrentSelection) -> Self {
        match selection {
            CurrentSelection::Voxels(_) => Self::Voxel,
            CurrentSelection::Unit(_) => Self::Unit,
            CurrentSelection::None => Self::None,
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
        center: VoxelPos,
        /// The distance to each corner of the hexagon, in tiles
        radius: u32,
    },
    /// A discretized line
    Line {
        /// The start of the line
        start: VoxelPos,
    },
}

impl SelectionState {
    /// Determine what selection state should be used this frame based on player actions
    fn compute(
        &mut self,
        tool: &Tool,
        actions: &ActionState<PlayerAction>,
        hovered_tile: VoxelPos,
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
            let radius = hovered_tile.hex.unsigned_distance_to(center.hex);

            SelectionShape::Area { center, radius }
        } else {
            SelectionShape::Single
        };

        self.action = match self.shape {
            SelectionShape::Single => {
                if actions.pressed(UseTool) {
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
                if actions.just_released(UseTool) {
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
        if !tool.is_empty() && !self.multiple {
            self.action = SelectionAction::Preview;
        }
    }
}

/// Determine what should be selected based on player inputs.
fn set_selection(
    tool: Res<Tool>,
    mut current_selection: ResMut<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    mut hovered_tiles: ResMut<HoveredTiles>,
    mut selection_state: ResMut<SelectionState>,
    mut last_tile_selected: Local<Option<VoxelPos>>,
    map_geometry: Res<MapGeometry>,
) {
    // Cast to ordinary references for ease of use
    let actions = &*actions;
    let cursor_pos = &*cursor_pos;
    let map_geometry = &*map_geometry;

    let Some(hovered_tile) = cursor_pos.maybe_voxel_pos() else {
        return;
    };

    // Compute how we should handle the selection based on the actions of the player
    selection_state.compute(&tool, actions, hovered_tile);

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
                (*last_tile_selected, cursor_pos.maybe_voxel_pos())
            {
                last_pos == current_pos
            } else {
                false
            };
            // Update the cache
            *last_tile_selected = cursor_pos.maybe_voxel_pos();

            if same_tile_as_last_time
                && !selection_state.multiple
                && actions.just_pressed(PlayerAction::UseTool)
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
                CurrentSelection::Voxels(ref mut selected_voxels) => {
                    if let Some(hovered_tile) = cursor_pos.maybe_voxel_pos() {
                        selected_voxels.remove_from_selection(
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
                CurrentSelection::Voxels(ref mut selected_voxels) => {
                    if let Some(hovered_tile) = cursor_pos.maybe_voxel_pos() {
                        selected_voxels.remove_from_selection(
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
    mut terrain_query: Query<(&VoxelPos, &mut ObjectInteraction)>,
) {
    if current_selection.is_changed() || hovered_tiles.is_changed() {
        for (voxel_pos, mut object_interaction) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&voxel_pos.hex);
            let selected = if let CurrentSelection::Voxels(selected_voxels) = &*current_selection {
                selected_voxels.contains(voxel_pos)
            } else {
                false
            };

            *object_interaction = ObjectInteraction::new(hovered, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::utils::HashSet;

    use super::SelectedVoxels;
    use crate::{
        enum_iter::IterableEnum,
        geometry::VoxelPos,
        player_interaction::{
            picking::CursorPos,
            selection::{CurrentSelection, SelectionVariant},
        },
    };

    #[test]
    fn simple_selection() {
        let mut selected_voxels = SelectedVoxels::default();
        let voxel_pos = VoxelPos::default();

        selected_voxels.insert(voxel_pos);
        assert!(selected_voxels.contains(&voxel_pos));
        assert!(!selected_voxels.is_empty());
        assert_eq!(selected_voxels.selected.len(), 1);

        selected_voxels.remove(&voxel_pos);
        assert!(!selected_voxels.contains(&voxel_pos));
        assert!(selected_voxels.is_empty());
        assert_eq!(selected_voxels.len(), 0);
    }

    #[test]
    fn multi_select() {
        let mut selected_voxels = SelectedVoxels::default();

        selected_voxels.insert(VoxelPos::from_xy(1, 1));
        // Intentionally doubled
        selected_voxels.insert(VoxelPos::from_xy(1, 1));
        selected_voxels.insert(VoxelPos::from_xy(2, 2));
        selected_voxels.insert(VoxelPos::from_xy(3, 3));

        assert_eq!(selected_voxels.selected.len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_voxels = SelectedVoxels::default();
        selected_voxels.insert(VoxelPos::from_xy(1, 1));
        selected_voxels.insert(VoxelPos::from_xy(2, 2));
        selected_voxels.insert(VoxelPos::from_xy(3, 3));

        assert_eq!(selected_voxels.selected.len(), 3);
        selected_voxels.clear();
        assert_eq!(selected_voxels.selected.len(), 0);
    }

    #[test]
    fn relevant_tiles_returns_cursor_pos_with_empty_selection() {
        let cursor_pos = CursorPos::new(VoxelPos::from_xy(24, 7));
        let mut cursor_pos_selected = SelectedVoxels::default();
        cursor_pos_selected.insert(cursor_pos.maybe_voxel_pos().unwrap());

        assert_eq!(
            CurrentSelection::None.relevant_tiles(&cursor_pos),
            cursor_pos_selected
        );

        assert_eq!(
            CurrentSelection::Voxels(SelectedVoxels::default()).relevant_tiles(&cursor_pos),
            cursor_pos_selected
        );
    }

    #[test]
    fn next_never_returns_none() {
        for variant in SelectionVariant::variants() {
            assert!(variant.next() != SelectionVariant::None);
        }
    }

    #[test]
    fn next_returns_all_variants_exactly_once() {
        let mut seen = HashSet::new();
        for variant in SelectionVariant::variants() {
            if variant == SelectionVariant::None {
                continue;
            }

            assert!(!seen.contains(&variant));
            seen.insert(variant);
        }
        assert_eq!(seen.len(), SelectionVariant::variants().len() - 1);
    }

    #[test]
    fn next_cycles_back_to_start() {
        for variant in SelectionVariant::variants() {
            if variant == SelectionVariant::None {
                continue;
            }

            let mut working_variant = variant;
            for _ in 0..(SelectionVariant::variants().len() - 1) {
                dbg!(working_variant);
                working_variant = working_variant.next();
            }

            assert_eq!(variant, working_variant);
        }
    }

    #[test]
    fn next_never_returns_self() {
        for variant in SelectionVariant::variants() {
            assert!(variant.next() != variant);
        }
    }

    #[test]
    fn cycle_end_with_self() {
        for variant in SelectionVariant::variants() {
            if variant == SelectionVariant::None {
                continue;
            }

            let cycle = variant.cycle();
            assert_eq!(
                cycle.iter().last().copied().unwrap(),
                variant,
                "{variant:?}'s cycle was {cycle:?}"
            );
        }
    }

    #[test]
    fn cycle_is_right_length() {
        for variant in SelectionVariant::variants() {
            let cycle = variant.cycle();
            if variant == SelectionVariant::None {
                assert_eq!(cycle.len(), 0);
            } else {
                // -1 for None
                assert_eq!(cycle.len(), SelectionVariant::variants().len() - 1);
            }
        }
    }
}
