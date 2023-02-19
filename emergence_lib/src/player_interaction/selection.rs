//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::terrain::TerrainHandles, simulation::geometry::TilePos, terrain::Terrain,
};

use crate as emergence_lib;

use super::{cursor::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionRadius>()
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
    /// Tiles that are hovered over
    hovered: HashSet<TilePos>,
    /// The radius of tiles to be selected
    radius: u32,
    /// The tile selected at the start of the action
    start: Option<TilePos>,
    /// The tiles selected at the start of this operation.
    ///
    /// Used to revert to the previous selection if an operation is cancelled.
    initial_selection: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    fn add_tile(&mut self, tile_pos: TilePos) {
        self.selected.insert(tile_pos);
    }

    /// Deselects a single tile
    fn remove_tile(&mut self, tile_pos: TilePos) {
        self.selected.remove(&tile_pos);
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

    /// Starts an action, caching the appropriate state.
    pub(super) fn begin_action(&mut self, hovered_tile: TilePos) {
        self.start = Some(hovered_tile);
        self.initial_selection = self.selected.clone();
    }

    /// Concludes an action, flushing any stored state.
    pub(super) fn end_action(&mut self) {
        self.start = None;
        self.initial_selection = HashSet::new();
    }

    /// Handles all of the logic needed to add tiles to the selection.
    pub(super) fn add_to_selection(
        existing_selection: Option<SelectedTiles>,
        hovered_tile: TilePos,
        player_actions: &ActionState<PlayerAction>,
    ) -> SelectedTiles {
        let mut selected_tiles = existing_selection.unwrap_or_default();

        let selection_region =
            selected_tiles.compute_selection_region(hovered_tile, player_actions);
        selected_tiles.selected =
            HashSet::from_iter(selected_tiles.selected.union(&selection_region).copied());

        selected_tiles
    }

    /// Handles all of the logic needed to remove tiles from the selection.
    pub(super) fn remove_from_selection(
        &mut self,
        hovered_tile: TilePos,
        player_actions: &ActionState<PlayerAction>,
    ) {
        if player_actions.released(PlayerAction::Multiple) {
            self.clear_selection()
        } else {
            let selection_region = self.compute_selection_region(hovered_tile, player_actions);

            self.selected =
                HashSet::from_iter(self.selected.difference(&selection_region).copied());
        }
    }

    fn compute_selection_region(
        &self,
        hovered_tile: TilePos,
        player_actions: &ActionState<PlayerAction>,
    ) -> HashSet<TilePos> {
        HashSet::new()
    }

    pub(super) fn show_hover_region(
        &mut self,
        hovered_tile: TilePos,
        player_actions: &ActionState<PlayerAction>,
    ) {
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

/// The radius of tiles that is selected at once
#[derive(Resource, Debug, Default)]
struct SelectionRadius {
    /// The number of tiles away from the central tile that are selected.
    ///
    /// 0 selects 1 tile, 1 selects 7 tiles and so on.
    size: u32,
}

impl SelectionRadius {
    /// The maximum radius of tiles that can be selected at once.
    ///
    /// This exists to avoid silly or performance degrading
    const MAX_SIZE: u32 = 20;
}

/// Sets the radius of "brush" used to select tiles.
fn set_selection_radius(
    mut selection_radius: ResMut<SelectionRadius>,
    actions: Res<ActionState<PlayerAction>>,
) {
    if actions.just_pressed(PlayerAction::IncreaseSelectionRadius) {
        selection_radius.size = (selection_radius.size + 1).min(SelectionRadius::MAX_SIZE);
    }

    if actions.just_pressed(PlayerAction::DecreaseSelectionRadius) {
        selection_radius.size = selection_radius.size.saturating_sub(1);
    }
}

/// Integrates user input into tile selection actions to let other systems handle what happens to a selected tile
#[allow(clippy::too_many_arguments)]
fn select_tiles(
    cursor: Res<CursorPos>,
    mut current_selection: ResMut<CurrentSelection>,
    actions: Res<ActionState<PlayerAction>>,
    selection_radius: Res<SelectionRadius>,
) {
    if let Some(cursor_pos) = cursor.maybe_tile_pos() {
        let select = actions.pressed(PlayerAction::Select);
        let deselect = actions.pressed(PlayerAction::Deselect);

        let multiple = actions.pressed(PlayerAction::Multiple);
        let area = actions.pressed(PlayerAction::Area);
        let line = actions.pressed(PlayerAction::Line);
        let simple_deselect = deselect & !area & !multiple & !line;

        let mut selected_tiles =
            if let CurrentSelection::Terrain(existing_selection) = &*current_selection {
                existing_selection.clone()
            } else {
                SelectedTiles::default()
            };

        // Cache the starting state to make selections reversible
        if area & area_selection.initial_selection.is_none() {
            area_selection.begin(&selected_tiles, cursor_pos, &selection_radius);
        }

        if line & line_selection.initial_selection.is_none() {
            line_selection.begin(&selected_tiles, cursor_pos);
        }

        // Clean up state from area and line selections
        if !area {
            area_selection.finish();
        }

        if !line {
            line_selection.finish();
        }

        let mut changing_selection_radius = false;

        // Compute the center and radius
        let (center, radius) = if area {
            let center = area_selection.center.unwrap();

            // Don't mess with indicator for changing selection size
            let proposed_radius = cursor_pos.unsigned_distance_to(center.hex);
            if proposed_radius == 0 {
                changing_selection_radius = true;
            } else {
                area_selection.radius = proposed_radius;
            }

            (center, area_selection.radius)
        } else {
            (cursor_pos, selection_radius.size)
        };

        // Record which tiles should have the "hovered" effect
        selected_tiles.hovered.clear();
        if area & !changing_selection_radius {
            selected_tiles.hovered.insert(center);
            let ring = center.hex.ring(radius);
            for hex in ring {
                selected_tiles.hovered.insert(TilePos { hex });
            }
        } else if line {
            let line_hexes = line_selection.draw_line(cursor_pos, radius);
            selected_tiles.hovered.extend(line_hexes);
        } else {
            let hexagon = hexagon(center.hex, radius);
            for hex in hexagon {
                selected_tiles.hovered.insert(TilePos { hex });
            }
        }

        // Don't attempt to handle conflicting inputs.
        if select & deselect {
            return;
        }

        // Clear the selection
        if simple_deselect | (select & !multiple) {
            selected_tiles.clear_selection()
        }

        // Actually select tiles
        if line {
            if actions.just_released(PlayerAction::Select) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                selected_tiles.selected.extend(line_hexes);
                line_selection.start = Some(cursor_pos);
            } else if actions.just_released(PlayerAction::Deselect) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                for tile_pos in line_hexes {
                    selected_tiles.selected.remove(&tile_pos);
                }
                line_selection.start = Some(cursor_pos);
            }
        } else {
            if select {
                selected_tiles.select_hexagon(center, radius, true);
            }

            if deselect {
                selected_tiles.select_hexagon(center, radius, false);
            }
        }

        // Save the selected tiles as the current selection, if any were selected
        if !selected_tiles.is_empty() {
            *current_selection = CurrentSelection::Terrain(selected_tiles);
        }
    }
}

/// Shows which tiles are being hovered and selected.
fn display_tile_interactions(
    current_selection: Res<CurrentSelection>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TerrainHandles>,
) {
    if current_selection.is_changed() {
        if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
            // PERF: We should probably avoid a linear scan over all tiles here
            for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
                let hovered = selected_tiles.hovered.contains(&tile_pos);
                let selected = selected_tiles.selected.contains(&tile_pos);

                *material = materials.get_material(terrain, hovered, selected);
            }
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

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use self::structure::*;
use self::terrain::*;
use self::unit::*;

use crate::player_interaction::{cursor::CursorPos, InteractionSystem};
use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

use super::selection::SelectedTiles;
use super::PlayerAction;

/// Display detailed info on hover.
pub(super) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building DetailsPlugin...");

        app.init_resource::<CurrentSelection>()
            .init_resource::<SelectionDetails>()
            .add_system(
                set_selection
                    .label(InteractionSystem::HoverDetails)
                    .after(InteractionSystem::ComputeCursorPos)
                    .before(get_details),
            )
            .add_system(
                get_details
                    .label(InteractionSystem::HoverDetails)
                    .after(InteractionSystem::SelectTiles),
            );
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
    fn update_from_cursor_pos(&mut self, cursor_pos: &CursorPos) {
        *self = if let Some(unit_entity) = cursor_pos.maybe_unit() {
            CurrentSelection::Unit(unit_entity)
        } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
            CurrentSelection::Structure(structure_entity)
        } else if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
            CurrentSelection::Terrain(todo!())
        } else {
            CurrentSelection::None
        }
    }

    fn cycle_selection(&mut self, cursor_pos: &CursorPos) {
        *self = match self {
            CurrentSelection::None => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
                    CurrentSelection::Terrain(todo!())
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Structure(_) => {
                if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
                    CurrentSelection::Terrain(todo!())
                } else if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Terrain(_) => {
                if let Some(unit_entity) = cursor_pos.maybe_unit() {
                    CurrentSelection::Unit(unit_entity)
                } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
                    CurrentSelection::Terrain(todo!())
                } else {
                    CurrentSelection::None
                }
            }
            CurrentSelection::Unit(_) => {
                if let Some(structure_entity) = cursor_pos.maybe_structure() {
                    CurrentSelection::Structure(structure_entity)
                } else if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
                    CurrentSelection::Terrain(todo!())
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

/// Determine what should be selected
fn set_selection(
    mut current_selection: ResMut<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    player_actions: Res<ActionState<PlayerAction>>,
    mut last_tile_selected: Local<Option<TilePos>>,
) {
    if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
        if let CurrentSelection::Terrain(selected_tiles) = *current_selection {
            // Clear the existing selection unless we're using multi-select
            if (player_actions.pressed(PlayerAction::Select)
                || player_actions.pressed(PlayerAction::Deselect))
                && player_actions.released(PlayerAction::Multiple)
            {
                selected_tiles.clear_selection()
            } else if player_actions.just_pressed(PlayerAction::Line)
                || player_actions.just_pressed(PlayerAction::Area)
            {
                // Cache any necessary state for terrain selection
                selected_tiles.begin_action(hovered_tile);
            }
        }
    }

    if player_actions.pressed(PlayerAction::Select) {
        let same_tile_as_last_time = if let (Some(last_pos), Some(current_pos)) =
            (*last_tile_selected, cursor_pos.maybe_tile_pos())
        {
            last_pos == current_pos
        } else {
            false
        };
        *last_tile_selected = cursor_pos.maybe_tile_pos();

        if same_tile_as_last_time {
            // Cycle through the options: unit -> structure -> terrain -> unit
            // Fall back to self if nothing else is there, to debounce inputs a bit
            // Don't cycle back to None, as users can just deselect instead.
            current_selection.cycle_selection(&*cursor_pos)
        } else {
            current_selection.update_from_cursor_pos(&*cursor_pos)
        }
    } else if player_actions.pressed(PlayerAction::Deselect) {
        *last_tile_selected = None;

        match &*current_selection {
            CurrentSelection::Terrain(selected_tiles) => {
                if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                    selected_tiles.remove_from_selection(hovered_tile, &*player_actions);
                }
            }
            _ => *current_selection = CurrentSelection::None,
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
mod structure {
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
mod terrain {
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
mod unit {
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
