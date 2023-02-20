//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use self::structure_details::*;
use self::terrain_details::*;
use self::unit_details::*;

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use leafwing_input_manager::prelude::ActionState;

use crate::simulation::geometry::MapGeometry;
use crate::{
    asset_management::terrain::TerrainHandles, simulation::geometry::TilePos, terrain::Terrain,
};

use crate as emergence_lib;

use super::{cursor::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSelection>()
            .init_resource::<SelectionDetails>()
            .init_resource::<SelectionData>()
            .init_resource::<HoveredTiles>()
            .add_system(
                set_selection
                    .label(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos)
                    .before(get_details),
            )
            .add_system(
                get_details
                    .label(InteractionSystem::HoverDetails)
                    .after(InteractionSystem::SelectTiles),
            )
            .add_system(set_selection_radius)
            .add_system(
                display_tile_interactions
                    .after(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            );
    }
}

/// The set of tiles that is currently selected
#[derive(Debug, Default, Clone)]
pub(crate) struct SelectedTiles {
    /// Actively selected tiles
    selected: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    #[cfg(test)]
    fn add_tile(&mut self, tile_pos: TilePos) {
        self.selected.insert(tile_pos);
    }

    /// Deselects a single tile
    #[cfg(test)]
    fn remove_tile(&mut self, tile_pos: TilePos) {
        self.selected.remove(&tile_pos);
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
    pub(super) fn selection(&self) -> &HashSet<TilePos> {
        &self.selected
    }

    /// Are any tiles selected?
    pub(super) fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Is the given tile in the selection?
    #[cfg(test)]
    fn contains_tile(&self, tile_pos: TilePos) -> bool {
        self.selected.contains(&tile_pos)
    }

    /// Handles all of the logic needed to add tiles to the selection.
    fn add_to_selection(
        &mut self,
        hovered_tile: TilePos,
        selection_state: SelectionState,
        select_multiple: bool,
        radius: u32,
    ) {
        let selection_region = self.compute_selection_region(hovered_tile, selection_state, radius);

        self.selected = match select_multiple {
            true => HashSet::from_iter(self.selected.union(&selection_region).copied()),
            false => selection_region,
        }
    }

    /// Handles all of the logic needed to remove tiles from the selection.
    fn remove_from_selection(
        &mut self,
        hovered_tile: TilePos,
        selection_state: SelectionState,
        select_multiple: bool,
        radius: u32,
    ) {
        if select_multiple {
            let selection_region =
                self.compute_selection_region(hovered_tile, selection_state, radius);

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
        selection_state: SelectionState,
        radius: u32,
    ) -> HashSet<TilePos> {
        match selection_state.mode() {
            SelectionMode::Single => SelectedTiles::draw_hexagon(hovered_tile, radius),
            SelectionMode::Area => SelectedTiles::draw_hexagon(
                selection_state.start().unwrap(),
                selection_state.radius().unwrap(),
            ),
            SelectionMode::Line => {
                SelectedTiles::draw_line(selection_state.start().unwrap(), hovered_tile, radius)
            }
        }
    }
}

/// The set of tiles that are being hovered
#[derive(Resource, Debug, Default, Deref, DerefMut)]
struct HoveredTiles {
    /// The set of tiles that are hovered over
    hovered: HashSet<TilePos>,
}

impl HoveredTiles {
    /// Updates the set of hovered actions based on the current cursor position and player inputs.
    pub(super) fn update(
        &mut self,
        hovered_tile: TilePos,
        selection_state: SelectionState,
        radius: u32,
    ) {
        self.hovered = match selection_state.mode() {
            SelectionMode::Single => SelectedTiles::draw_hexagon(hovered_tile, radius),
            SelectionMode::Area => {
                let mut set = SelectedTiles::draw_ring(
                    selection_state.start().unwrap(),
                    selection_state.radius().unwrap(),
                );
                // Also show center of ring for clarity.
                set.insert(hovered_tile);
                set
            }
            SelectionMode::Line => {
                SelectedTiles::draw_line(selection_state.start().unwrap(), hovered_tile, radius)
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
    /// The material used by objects that are being interacted with.
    pub(crate) fn material(&self) -> Option<StandardMaterial> {
        use crate::asset_management::palette::{
            HOVER_COLOR, SELECTION_AND_HOVER_COLOR, SELECTION_COLOR,
        };

        let maybe_color = match self {
            ObjectInteraction::Selected => Some(SELECTION_COLOR),
            ObjectInteraction::Hovered => Some(HOVER_COLOR),
            ObjectInteraction::HoveredAndSelected => Some(SELECTION_AND_HOVER_COLOR),
            ObjectInteraction::None => None,
        };

        maybe_color.map(|base_color| StandardMaterial {
            base_color,
            ..Default::default()
        })
    }

    /// The material used by ghosts and previews, based on their interaction
    pub(crate) fn ghost_material(&self) -> Option<StandardMaterial> {
        use crate::asset_management::palette::{GHOST_COLOR, PREVIEW_COLOR, SELECTED_GHOST_COLOR};

        let maybe_color = match self {
            ObjectInteraction::Selected => Some(SELECTED_GHOST_COLOR),
            ObjectInteraction::Hovered => Some(PREVIEW_COLOR),
            ObjectInteraction::HoveredAndSelected => None,
            ObjectInteraction::None => Some(GHOST_COLOR),
        };

        maybe_color.map(|base_color| StandardMaterial {
            base_color,
            ..Default::default()
        })
    }
}

/// The paramaters and cached state used to handle selecting and hovering tiles.
///
/// This is stored outside of [`SelectedTiles`] in order to persist nicely across different selection modes,
/// and preview selection areas regardless of selection mode.
#[derive(Resource, Debug, Default)]
struct SelectionData {
    /// The number of tiles away from the central tile that are selected.
    ///
    /// 0 selects 1 tile, 1 selects 7 tiles and so on.
    radius: u32,
}

impl SelectionData {
    /// The maximum reasanably allowable selection size
    const MAX_SIZE: u32 = 10;
}

/// Sets the radius of "brush" used to select tiles.
fn set_selection_radius(
    mut selection_data: ResMut<SelectionData>,
    actions: Res<ActionState<PlayerAction>>,
) {
    if actions.just_pressed(PlayerAction::IncreaseSelectionRadius) {
        selection_data.radius = (selection_data.radius + 1).min(SelectionData::MAX_SIZE);
    }

    if actions.just_pressed(PlayerAction::DecreaseSelectionRadius) {
        selection_data.radius = selection_data.radius.saturating_sub(1);
    }
}

/// Shows which tiles are being hovered and selected.
fn display_tile_interactions(
    current_selection: Res<CurrentSelection>,
    hovered_tiles: Res<HoveredTiles>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TerrainHandles>,
) {
    if current_selection.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&tile_pos);
            let selected = if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
                selected_tiles.selected.contains(&tile_pos)
            } else {
                false
            };

            *material = materials.get_material(terrain, hovered, selected);
        }
    }
}

/// The game object(s) currently selected for inspection.
#[derive(Resource, Debug, Default)]
pub(crate) enum CurrentSelection {
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
    /// Determines the selection based on the cursor information.
    ///
    /// This handles the simple case, when we're selecting a new tile.
    /// Ordinarily, just prioritize units > structures > terrain
    fn update_from_cursor_pos(
        &mut self,
        cursor_pos: &CursorPos,
        hovered_tile: TilePos,
        selection_state: SelectionState,
        select_multiple: bool,
        radius: u32,
    ) {
        *self = if select_multiple {
            if let CurrentSelection::Terrain(existing_selection) = self {
                existing_selection.add_to_selection(
                    hovered_tile,
                    selection_state,
                    select_multiple,
                    radius,
                );
                CurrentSelection::Terrain(existing_selection.clone())
            } else {
                let mut selected_tiles = SelectedTiles::default();
                selected_tiles.add_to_selection(
                    hovered_tile,
                    selection_state,
                    select_multiple,
                    radius,
                );
                CurrentSelection::Terrain(selected_tiles)
            }
        } else {
            if let Some(unit_entity) = cursor_pos.maybe_unit() {
                CurrentSelection::Unit(unit_entity)
            } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                CurrentSelection::Structure(structure_entity)
            } else {
                if let CurrentSelection::Terrain(existing_selection) = self {
                    existing_selection.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
                    CurrentSelection::Terrain(existing_selection.clone())
                } else {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
                    CurrentSelection::Terrain(selected_tiles)
                }
            }
        }
    }

    /// Cycles through game objects on the same tile.
    ///
    /// The order is units -> structures -> terrain -> units.
    /// If a higher priority option is missing, later options in the chain are searched.
    /// If none of the options can be found, the selection is cleared completely.
    fn cycle_selection(
        &mut self,
        cursor_pos: &CursorPos,
        selection_state: SelectionState,
        select_multiple: bool,
        radius: u32,
    ) {
        *self = match self {
            CurrentSelection::None => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
                    CurrentSelection::Terrain(selected_tiles)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Structure(_) => {
                if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
                    CurrentSelection::Terrain(selected_tiles)
                } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Terrain(existing_selection) => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    existing_selection.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
                    CurrentSelection::Terrain(existing_selection.clone())
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Unit(_) => {
                if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(
                        hovered_tile,
                        selection_state,
                        select_multiple,
                        radius,
                    );
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

impl CurrentSelection {
    /// Is anything currently selected?
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            CurrentSelection::Structure(_) => false,
            CurrentSelection::Terrain(selected_tiles) => selected_tiles.is_empty(),
            CurrentSelection::Unit(_) => false,
            CurrentSelection::None => true,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
enum SelectionState {
    PreviewLine {
        start: TilePos,
    },
    SelectLine {
        start: TilePos,
    },
    DeselectLine {
        start: TilePos,
    },
    PreviewArea {
        start: TilePos,
        radius: u32,
    },
    SelectArea {
        start: TilePos,
        radius: u32,
    },
    DeselectArea {
        start: TilePos,
        radius: u32,
    },
    #[default]
    PreviewSingle,
    SelectSingle,
    DeselectSingle,
}

enum SelectionMode {
    Area,
    Line,
    Single,
}

impl SelectionState {
    fn compute(&mut self, actions: &ActionState<PlayerAction>, hovered_tile: TilePos) {
        use PlayerAction::*;

        let start = self.start().unwrap_or(hovered_tile);

        *self = if actions.pressed(Line) {
            if actions.just_released(Select) {
                SelectionState::SelectLine { start }
            } else if actions.just_released(Deselect) {
                SelectionState::DeselectLine { start }
            } else {
                SelectionState::PreviewLine { start }
            }
        } else if actions.pressed(Area) {
            let radius = hovered_tile.unsigned_distance_to(start.hex);

            if actions.just_released(Select) {
                SelectionState::SelectArea { start, radius }
            } else if actions.just_released(Deselect) {
                SelectionState::DeselectArea { start, radius }
            } else {
                SelectionState::PreviewArea { start, radius }
            }
        } else {
            if actions.pressed(Select) {
                SelectionState::SelectSingle
            // Don't repeatedly trigger deselect to avoid
            } else if actions.just_pressed(Deselect) {
                SelectionState::DeselectSingle
            } else {
                SelectionState::PreviewSingle
            }
        };
    }

    fn start(&self) -> Option<TilePos> {
        match *self {
            SelectionState::PreviewLine { start } => Some(start),
            SelectionState::SelectLine { start } => Some(start),
            SelectionState::DeselectLine { start } => Some(start),
            SelectionState::PreviewArea { start, .. } => Some(start),
            SelectionState::SelectArea { start, .. } => Some(start),
            SelectionState::DeselectArea { start, .. } => Some(start),
            SelectionState::PreviewSingle => None,
            SelectionState::SelectSingle => None,
            SelectionState::DeselectSingle => None,
        }
    }

    fn radius(&self) -> Option<u32> {
        match *self {
            SelectionState::PreviewLine { .. } => None,
            SelectionState::SelectLine { .. } => None,
            SelectionState::DeselectLine { .. } => None,
            SelectionState::PreviewArea { radius, .. } => Some(radius),
            SelectionState::SelectArea { radius, .. } => Some(radius),
            SelectionState::DeselectArea { radius, .. } => Some(radius),
            SelectionState::PreviewSingle => None,
            SelectionState::SelectSingle => None,
            SelectionState::DeselectSingle => None,
        }
    }

    fn mode(&self) -> SelectionMode {
        match *self {
            SelectionState::PreviewLine { .. } => SelectionMode::Line,
            SelectionState::SelectLine { .. } => SelectionMode::Line,
            SelectionState::DeselectLine { .. } => SelectionMode::Line,
            SelectionState::PreviewArea { .. } => SelectionMode::Area,
            SelectionState::SelectArea { .. } => SelectionMode::Area,
            SelectionState::DeselectArea { .. } => SelectionMode::Area,
            SelectionState::PreviewSingle => SelectionMode::Single,
            SelectionState::SelectSingle => SelectionMode::Single,
            SelectionState::DeselectSingle => SelectionMode::Single,
        }
    }
}

/// Determine what should be selected based on player inputs.
fn set_selection(
    mut current_selection: ResMut<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    actions: Res<ActionState<PlayerAction>>,
    mut selection_data: ResMut<SelectionData>,
    mut hovered_tiles: ResMut<HoveredTiles>,
    mut selection_state: Local<SelectionState>,
    mut last_tile_selected: Local<Option<TilePos>>,
) {
    // Cast to ordinary references for ease of use
    let actions = &*actions;
    let cursor_pos = &*cursor_pos;
    let selection_data = &mut *selection_data;
    let hovered_tile = cursor_pos.maybe_tile_pos().unwrap_or_default();
    let select_multiple = actions.pressed(PlayerAction::Multiple);
    let radius = selection_data.radius;

    // Compute how we should handle the selection based on the actions of the player
    selection_state.compute(actions, hovered_tile);

    // Update hovered tiles
    hovered_tiles.update(hovered_tile, *selection_state, radius);

    // Select and deselect tiles
    match *selection_state {
        // No need to do work here, hovered tiles are always computed
        SelectionState::PreviewLine { .. }
        | SelectionState::PreviewArea { .. }
        | SelectionState::PreviewSingle => (),
        SelectionState::SelectLine { .. } => {
            current_selection.update_from_cursor_pos(
                cursor_pos,
                hovered_tile,
                *selection_state,
                select_multiple,
                radius,
            );
            // Let players chain lines head to tail nicely
            *selection_state = SelectionState::PreviewLine {
                start: hovered_tile,
            };
        }
        SelectionState::SelectArea { .. } => {
            current_selection.update_from_cursor_pos(
                cursor_pos,
                hovered_tile,
                *selection_state,
                select_multiple,
                radius,
            );
        }
        SelectionState::SelectSingle => {
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
                && !select_multiple
                && actions.just_pressed(PlayerAction::Select)
            {
                current_selection.cycle_selection(
                    cursor_pos,
                    *selection_state,
                    select_multiple,
                    radius,
                )
            } else {
                current_selection.update_from_cursor_pos(
                    cursor_pos,
                    hovered_tile,
                    *selection_state,
                    select_multiple,
                    radius,
                )
            }
        }
        SelectionState::DeselectSingle
        | SelectionState::DeselectArea { .. }
        | SelectionState::DeselectLine { .. } => match &mut *current_selection {
            CurrentSelection::Terrain(ref mut selected_tiles) => {
                if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    selected_tiles.remove_from_selection(
                        hovered_tile,
                        *selection_state,
                        select_multiple,
                        radius,
                    );
                }
            }
            _ => *current_selection = CurrentSelection::None,
        },
    }
}

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default)]
pub(crate) enum SelectionDetails {
    /// A structure is selected
    Structure(StructureDetails),
    /// A tile is selected.
    Terrain(TerrainDetails),
    /// A unit is selected
    Unit(UnitDetails),
    /// Nothing is selected
    #[default]
    None,
}

/// Get details about the hovered entity.
fn get_details(
    selection_type: Res<CurrentSelection>,
    mut selection_details: ResMut<SelectionDetails>,
    terrain_query: Query<TerrainDetailsQuery>,
    unit_query: Query<UnitDetailsQuery>,
    structure_query: Query<StructureDetailsQuery>,
    map_geometry: Res<MapGeometry>,
) {
    *selection_details = match &*selection_type {
        CurrentSelection::Terrain(selected_tiles) => {
            // FIXME: display info about multiple tiles correctly
            if let Some(tile_pos) = selected_tiles.selection().iter().next() {
                let terrain_entity = map_geometry.terrain_index.get(tile_pos).unwrap();
                let terrain_data = terrain_query.get(*terrain_entity).unwrap();

                SelectionDetails::Terrain(TerrainDetails {
                    terrain_type: *terrain_data.terrain_type,
                    tile_pos: *tile_pos,
                })
            } else {
                SelectionDetails::None
            }
        }
        CurrentSelection::Unit(unit_entity) => {
            let unit_query_item = unit_query.get(*unit_entity).unwrap();
            SelectionDetails::Unit(UnitDetails {
                unit_id: *unit_query_item.unit_id,
                tile_pos: *unit_query_item.tile_pos,
                held_item: unit_query_item.held_item.clone(),
                goal: unit_query_item.goal.clone(),
                action: unit_query_item.action.clone(),
            })
        }
        CurrentSelection::Structure(structure_entity) => {
            let structure_query_item = structure_query.get(*structure_entity).unwrap();

            let crafting_details = if let Some((input, output, recipe, state, timer)) =
                structure_query_item.crafting
            {
                Some(CraftingDetails {
                    input_inventory: input.inventory.clone(),
                    output_inventory: output.inventory.clone(),
                    active_recipe: *recipe.recipe_id(),
                    state: state.clone(),
                    timer: timer.timer().clone(),
                })
            } else {
                None
            };

            SelectionDetails::Structure(StructureDetails {
                tile_pos: *structure_query_item.tile_pos,
                structure_id: *structure_query_item.structure_id,
                crafting_details,
            })
        }
        CurrentSelection::None => SelectionDetails::None,
    };
}

/// Details for structures
mod structure_details {
    use bevy::{
        ecs::{prelude::*, query::WorldQuery},
        time::Timer,
    };

    use core::fmt::Display;

    use crate::{
        items::{inventory::Inventory, recipe::RecipeId},
        simulation::geometry::TilePos,
        structures::{
            crafting::{ActiveRecipe, CraftTimer, CraftingState, InputInventory, OutputInventory},
            StructureId,
        },
    };

    /// Data needed to populate [`StructureDetails`].
    #[derive(WorldQuery)]
    pub(super) struct StructureDetailsQuery {
        /// The type of structure
        pub(super) structure_id: &'static StructureId,
        /// The tile position of this structure
        pub(crate) tile_pos: &'static TilePos,
        /// Crafting-related components
        pub(super) crafting: Option<(
            &'static InputInventory,
            &'static OutputInventory,
            &'static ActiveRecipe,
            &'static CraftingState,
            &'static CraftTimer,
        )>,
    }

    /// Detailed info about a given structure.
    #[derive(Debug)]
    pub(crate) struct StructureDetails {
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: StructureId,
        /// If this organism is crafting something, the details about that.
        pub(crate) crafting_details: Option<CraftingDetails>,
    }

    impl Display for StructureDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let structure_id = &self.structure_id;
            let tile_pos = &self.tile_pos;

            let basic_details = format!(
                "Structure type: {structure_id}
Tile: {tile_pos}"
            );

            let crafting_details = if let Some(crafting) = &self.crafting_details {
                format!("{crafting}")
            } else {
                String::default()
            };

            write!(f, "{basic_details}\n{crafting_details}")
        }
    }

    /// The details about crafting processes.
    #[derive(Debug, Clone)]
    pub(crate) struct CraftingDetails {
        /// The inventory for the input items.
        pub(crate) input_inventory: Inventory,

        /// The inventory for the output items.
        pub(crate) output_inventory: Inventory,

        /// The recipe that's currently being crafted, if any.
        pub(crate) active_recipe: Option<RecipeId>,

        /// The state of the ongoing crafting process.
        pub(crate) state: CraftingState,

        /// The time remaining to finish crafting.
        pub(crate) timer: Timer,
    }

    impl Display for CraftingDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let input_inventory = &self.input_inventory;
            let output_inventory = &self.output_inventory;
            let crafting_state = &self.state;
            let time_remaining = self.timer.remaining_secs();
            let total_duration = self.timer.duration().as_secs_f32();

            let recipe_string = match &self.active_recipe {
                Some(recipe_id) => format!("{recipe_id}"),
                None => "None".to_string(),
            };

            write!(
                f,
                "Recipe: {recipe_string}
Input: {input_inventory}
{crafting_state}: {time_remaining:.1} s / {total_duration:.1} s
Output: {output_inventory}"
            )
        }
    }
}

/// Details for terrain
mod terrain_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};
    use std::fmt::Display;

    use crate::{simulation::geometry::TilePos, terrain::Terrain};

    /// Data needed to populate [`TerrainDetails`].
    #[derive(WorldQuery)]
    pub(super) struct TerrainDetailsQuery {
        /// The type of terrain
        pub(super) terrain_type: &'static Terrain,
    }

    /// Detailed info about a given unit.
    #[derive(Debug)]
    pub(crate) struct TerrainDetails {
        /// The type of unit
        pub(super) terrain_type: Terrain,
        /// The location of the tile
        pub(super) tile_pos: TilePos,
    }

    impl Display for TerrainDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let terrain_type = &self.terrain_type;
            let tile_pos = &self.tile_pos;

            write!(
                f,
                "Terrain type: {terrain_type}
Tile: {tile_pos}"
            )
        }
    }
}

/// Details for units
mod unit_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};
    use std::fmt::Display;

    use crate::{
        simulation::geometry::TilePos,
        units::{
            behavior::{CurrentAction, Goal},
            item_interaction::HeldItem,
            UnitId,
        },
    };

    /// Data needed to populate [`UnitDetails`].
    #[derive(WorldQuery)]
    pub(super) struct UnitDetailsQuery {
        /// The type of unit
        pub(super) unit_id: &'static UnitId,
        /// The current location
        pub(super) tile_pos: &'static TilePos,
        /// What's being carried
        pub(super) held_item: &'static HeldItem,
        /// What this unit is trying to acheive
        pub(super) goal: &'static Goal,
        /// What is currently being done
        pub(super) action: &'static CurrentAction,
    }

    /// Detailed info about a given unit.
    #[derive(Debug)]
    pub(crate) struct UnitDetails {
        /// The type of unit
        pub(super) unit_id: UnitId,
        /// The current location
        pub(super) tile_pos: TilePos,
        /// What's being carried
        pub(super) held_item: HeldItem,
        /// What this unit is trying to acheive
        pub(super) goal: Goal,
        /// What is currently being done
        pub(super) action: CurrentAction,
    }

    impl Display for UnitDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let unit_id = &self.unit_id;
            let tile_pos = &self.tile_pos;
            let held_item = &self.held_item;
            let goal = &self.goal;
            let action = &self.action;

            write!(
                f,
                "Unit type: {unit_id}
Tile: {tile_pos}
Holding: {held_item}
Goal: {goal}
Action: {action}"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedTiles;
    use crate::simulation::geometry::TilePos;

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
}
