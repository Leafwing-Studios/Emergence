//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use self::structure_details::*;
use self::terrain_details::*;
use self::unit_details::*;

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use leafwing_input_manager::prelude::ActionState;

use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

use crate as emergence_lib;

use super::{cursor::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSelection>()
            .init_resource::<SelectionDetails>()
            .init_resource::<SelectionState>()
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
            .add_system(update_selection_radius);
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

    /// Is the given tile in the selection?
    pub(crate) fn contains_tile(&self, tile_pos: TilePos) -> bool {
        self.selected.contains(&tile_pos)
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

    /// Handles all of the logic needed to add tiles to the selection.
    fn add_to_selection(
        &mut self,
        hovered_tile: TilePos,
        selection_state: SelectionState,
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
        selection_state: SelectionState,
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
        selection_state: SelectionState,
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
        .filter(|tile_pos| map_geometry.terrain_index.contains_key(tile_pos))
        .collect()
    }
}

/// The set of tiles that are being hovered
#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub(crate) struct HoveredTiles {
    /// The set of tiles that are hovered over
    hovered: HashSet<TilePos>,
}

impl HoveredTiles {
    /// Updates the set of hovered actions based on the current cursor position and player inputs.
    fn update(&mut self, hovered_tile: TilePos, selection_state: SelectionState) {
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
    /// Just select the terrain.
    fn select_terrain(
        &self,
        hovered_tile: TilePos,
        selection_state: SelectionState,
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
        selection_state: SelectionState,
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
    /// The order is units -> structures -> terrain -> units.
    /// If a higher priority option is missing, later options in the chain are searched.
    /// If none of the options can be found, the selection is cleared completely.
    fn cycle_selection(
        &mut self,
        cursor_pos: &CursorPos,
        selection_state: SelectionState,
        map_geometry: &MapGeometry,
    ) {
        *self = match self {
            CurrentSelection::None => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
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
            CurrentSelection::Structure(_) => {
                if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_to_selection(hovered_tile, selection_state, map_geometry);
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
                        map_geometry,
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

/// Tracks what should be done with the selection (and hovered tiles) this frame.
#[derive(Resource, Default, Debug, Clone, Copy)]
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
    fn compute(&mut self, actions: &ActionState<PlayerAction>, hovered_tile: TilePos) {
        use PlayerAction::*;

        if actions.pressed(Line) {
            let start = if let SelectionShape::Line { start } = self.shape {
                start
            } else {
                hovered_tile
            };

            self.shape = SelectionShape::Line { start };

            self.action = if actions.just_released(Select) {
                SelectionAction::Select
            } else if actions.just_released(Deselect) {
                SelectionAction::Deselect
            } else {
                SelectionAction::Preview
            }
        } else if actions.pressed(Area) {
            let center = if let SelectionShape::Area { center, .. } = self.shape {
                center
            } else {
                hovered_tile
            };
            let radius = hovered_tile.unsigned_distance_to(center.hex);

            self.shape = SelectionShape::Area { center, radius };

            self.action = if actions.just_released(Select) {
                SelectionAction::Select
            } else if actions.just_released(Deselect) {
                SelectionAction::Deselect
            } else {
                SelectionAction::Preview
            }
        } else {
            self.shape = SelectionShape::Single;

            self.action = if actions.pressed(Select) {
                SelectionAction::Select
            // Don't repeatedly trigger deselect to avoid
            } else if actions.just_pressed(Deselect) {
                SelectionAction::Deselect
            } else {
                SelectionAction::Preview
            }
        };

        self.multiple = actions.pressed(PlayerAction::Multiple);
    }
}

/// Determine what should be selected based on player inputs.
fn set_selection(
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

    let hovered_tile = cursor_pos.maybe_tile_pos().unwrap_or_default();

    // Compute how we should handle the selection based on the actions of the player
    selection_state.compute(actions, hovered_tile);

    // Update hovered tiles
    hovered_tiles.update(hovered_tile, *selection_state);

    // Select and deselect tiles
    match (selection_state.action, selection_state.shape) {
        // No need to do work here, hovered tiles are always computed
        (SelectionAction::Preview, _) => (),
        (SelectionAction::Select, SelectionShape::Line { .. }) => {
            current_selection.update_from_cursor_pos(
                cursor_pos,
                hovered_tile,
                *selection_state,
                map_geometry,
            );
            // Let players chain lines head to tail nicely
            selection_state.shape = SelectionShape::Line {
                start: hovered_tile,
            };
        }
        (SelectionAction::Select, SelectionShape::Area { .. }) => {
            current_selection.update_from_cursor_pos(
                cursor_pos,
                hovered_tile,
                *selection_state,
                map_geometry,
            );
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
                current_selection.cycle_selection(cursor_pos, *selection_state, map_geometry)
            } else if !same_tile_as_last_time {
                current_selection.update_from_cursor_pos(
                    cursor_pos,
                    hovered_tile,
                    *selection_state,
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
                            *selection_state,
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
                            *selection_state,
                            map_geometry,
                        );
                    }
                }
                _ => *current_selection = CurrentSelection::None,
            }

            // Let players chain lines head to tail nicely
            // Let players chain lines head to tail nicely
            selection_state.shape = SelectionShape::Line {
                start: hovered_tile,
            };
        }
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
                let terrain_query_item = terrain_query.get(*terrain_entity).unwrap();

                SelectionDetails::Terrain(TerrainDetails {
                    entity: terrain_query_item.entity,
                    terrain_type: *terrain_query_item.terrain_type,
                    tile_pos: *tile_pos,
                })
            } else {
                SelectionDetails::None
            }
        }
        CurrentSelection::Unit(unit_entity) => {
            let unit_query_item = unit_query.get(*unit_entity).unwrap();
            SelectionDetails::Unit(UnitDetails {
                entity: unit_query_item.entity,
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
                entity: structure_query_item.entity,
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
        /// The root entity
        pub(super) entity: Entity,
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
        /// The root entity
        pub(super) entity: Entity,
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: StructureId,
        /// If this organism is crafting something, the details about that.
        pub(crate) crafting_details: Option<CraftingDetails>,
    }

    impl Display for StructureDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let entity = self.entity;
            let structure_id = &self.structure_id;
            let tile_pos = &self.tile_pos;

            let basic_details = format!(
                "Entity: {entity:?}
Structure type: {structure_id}
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
        /// The root entity
        pub(super) entity: Entity,
        /// The type of terrain
        pub(super) terrain_type: &'static Terrain,
    }

    /// Detailed info about a given unit.
    #[derive(Debug)]
    pub(crate) struct TerrainDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of terrain
        pub(super) terrain_type: Terrain,
        /// The location of the tile
        pub(super) tile_pos: TilePos,
    }

    impl Display for TerrainDetails {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let entity = self.entity;
            let terrain_type = &self.terrain_type;
            let tile_pos = &self.tile_pos;

            write!(
                f,
                "Entity: {entity:?}
Terrain type: {terrain_type}
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
        /// The root entity
        pub(super) entity: Entity,
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
        /// The root entity
        pub(super) entity: Entity,
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
            let entity = self.entity;
            let unit_id = &self.unit_id;
            let tile_pos = &self.tile_pos;
            let held_item = &self.held_item;
            let goal = &self.goal;
            let action = &self.action;

            write!(
                f,
                "Entity: {entity:?}
Unit type: {unit_id}
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
