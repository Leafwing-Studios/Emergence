//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use self::ghost_details::*;
use self::organism_details::*;
use self::structure_details::*;
use self::terrain_details::*;
use self::unit_details::*;

use bevy::ecs::query::QueryEntityError;
use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use hexx::HexIterExt;
use leafwing_input_manager::prelude::ActionState;

use crate::asset_management::manifest::RecipeManifest;
use crate::asset_management::manifest::StructureManifest;
use crate::asset_management::manifest::UnitManifest;
use crate::signals::Signals;
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
                    .in_set(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos)
                    .before(InteractionSystem::HoverDetails),
            )
            .add_system(
                set_tile_interactions
                    .in_set(InteractionSystem::SelectTiles)
                    .after(set_selection),
            )
            .add_system(
                get_details
                    .pipe(clear_details_on_error)
                    .in_set(InteractionSystem::HoverDetails)
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
        .filter(|tile_pos| map_geometry.terrain_index.contains_key(tile_pos))
        .collect()
    }

    /// Fetches the entities that correspond to these tiles
    pub(crate) fn entities(&self, map_geometry: &MapGeometry) -> Vec<Entity> {
        self.selection()
            .iter()
            .map(|tile_pos| *map_geometry.terrain_index.get(tile_pos).unwrap())
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
        use crate::asset_management::palette::{
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
            CurrentSelection::Terrain(selected_tiles) => selected_tiles.clone(),
            _ => match cursor_pos.maybe_tile_pos() {
                Some(cursor_tile_pos) => {
                    let mut selected_tiles = SelectedTiles::default();
                    selected_tiles.add_tile(cursor_tile_pos);
                    selected_tiles
                }
                None => SelectedTiles::default(),
            },
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

impl CurrentSelection {
    /// Is anything currently selected?
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            CurrentSelection::None => true,
            CurrentSelection::Terrain(selected_tiles) => selected_tiles.is_empty(),
            _ => false,
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
    fn compute(&mut self, actions: &ActionState<PlayerAction>, hovered_tile: TilePos) {
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

    let Some(hovered_tile) = cursor_pos.maybe_tile_pos() else {return};

    // Compute how we should handle the selection based on the actions of the player
    selection_state.compute(actions, hovered_tile);

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

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default)]
pub(crate) enum SelectionDetails {
    /// A ghost is selected
    Ghost(GhostDetails),
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
#[allow(clippy::too_many_arguments)]
fn get_details(
    selection_type: Res<CurrentSelection>,
    mut selection_details: ResMut<SelectionDetails>,
    ghost_query: Query<GhostDetailsQuery>,
    organism_query: Query<OrganismDetailsQuery>,
    structure_query: Query<StructureDetailsQuery>,
    terrain_query: Query<TerrainDetailsQuery>,
    unit_query: Query<UnitDetailsQuery>,
    map_geometry: Res<MapGeometry>,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
    recipe_manifest: Res<RecipeManifest>,
    signals: Res<Signals>,
) -> Result<(), QueryEntityError> {
    *selection_details = match &*selection_type {
        CurrentSelection::Ghost(ghost_entity) => {
            let ghost_query_item = ghost_query.get(*ghost_entity)?;
            SelectionDetails::Ghost(GhostDetails {
                entity: *ghost_entity,
                tile_pos: *ghost_query_item.tile_pos,
                structure_id: *ghost_query_item.structure_id,
                input_inventory: ghost_query_item.input_inventory.clone(),
                crafting_state: ghost_query_item.crafting_state.clone(),
                active_recipe: ghost_query_item.active_recipe.clone(),
            })
        }
        CurrentSelection::Structure(structure_entity) => {
            let structure_query_item = structure_query.get(*structure_entity)?;

            let crafting_details =
                if let Some((input, output, active_recipe, workers_present, state)) =
                    structure_query_item.crafting
                {
                    let maybe_recipe_id = *active_recipe.recipe_id();
                    let recipe =
                        maybe_recipe_id.map(|recipe_id| recipe_manifest.get(recipe_id).clone());

                    Some(CraftingDetails {
                        input_inventory: input.inventory.clone(),
                        output_inventory: output.inventory.clone(),
                        recipe,
                        workers_present: workers_present.clone(),
                        state: state.clone(),
                    })
                } else {
                    None
                };

            // Not all structures are organisms
            let maybe_organism_details =
                organism_query
                    .get(*structure_entity)
                    .ok()
                    .map(|query_item| OrganismDetails {
                        prototypical_form: structure_manifest
                            .get(*structure_query_item.structure_id)
                            .organism_variety.as_ref()
                            .expect("All structures with organism components must be registered in the manifest as organisms")
                            .prototypical_form,
                        lifecycle: query_item.lifecycle.clone(),
                        energy_pool: query_item.energy_pool.clone(),
                    });

            SelectionDetails::Structure(StructureDetails {
                entity: structure_query_item.entity,
                tile_pos: *structure_query_item.tile_pos,
                structure_id: *structure_query_item.structure_id,
                crafting_details,
                maybe_organism_details,
                storage_inventory: structure_query_item.storage_inventory.cloned(),
                marked_for_removal: structure_query_item.marked_for_removal.is_some(),
            })
        }
        CurrentSelection::Terrain(selected_tiles) => {
            // FIXME: display info about multiple tiles correctly
            if let Some(tile_pos) = selected_tiles.selection().iter().next() {
                let terrain_entity = *map_geometry.terrain_index.get(tile_pos).unwrap();
                let terrain_query_item = terrain_query.get(terrain_entity)?;

                SelectionDetails::Terrain(TerrainDetails {
                    entity: terrain_entity,
                    terrain_id: *terrain_query_item.terrain_id,
                    tile_pos: *tile_pos,
                    height: *terrain_query_item.height,
                    signals: signals.all_signals_at_position(*tile_pos),
                    zoning: terrain_query_item.zoning.clone(),
                })
            } else {
                SelectionDetails::None
            }
        }
        CurrentSelection::Unit(unit_entity) => {
            let unit_query_item = unit_query.get(*unit_entity)?;
            // All units are organisms
            let organism_query_item = organism_query.get(*unit_entity)?;
            let organism_details = OrganismDetails {
                prototypical_form: unit_manifest
                    .get(*unit_query_item.unit_id)
                    .organism_variety()
                    .prototypical_form,
                lifecycle: organism_query_item.lifecycle.clone(),
                energy_pool: organism_query_item.energy_pool.clone(),
            };

            SelectionDetails::Unit(UnitDetails {
                entity: unit_query_item.entity,
                unit_id: *unit_query_item.unit_id,
                diet: unit_query_item.diet.clone(),
                tile_pos: *unit_query_item.tile_pos,
                held_item: unit_query_item.held_item.clone(),
                goal: unit_query_item.goal.clone(),
                action: unit_query_item.action.clone(),
                impatience_pool: unit_query_item.impatience_pool.clone(),
                organism_details,
            })
        }
        CurrentSelection::None => SelectionDetails::None,
    };

    Ok(())
}

/// If something went wrong in [`get_details`], clear the selection.
pub(crate) fn clear_details_on_error(
    In(result): In<Result<(), QueryEntityError>>,
    mut current_selection: ResMut<CurrentSelection>,
    mut selection_details: ResMut<SelectionDetails>,
) {
    if result.is_err() {
        *current_selection = CurrentSelection::None;
        *selection_details = SelectionDetails::None;
    }
}

/// Details for ghosts
mod ghost_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::{
            Id, ItemManifest, RecipeManifest, Structure, StructureManifest,
        },
        signals::Emitter,
        simulation::geometry::TilePos,
        structures::crafting::{ActiveRecipe, CraftingState, InputInventory},
    };

    /// Data needed to populate [`GhostDetails`].
    #[derive(WorldQuery)]
    pub(super) struct GhostDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of structure
        pub(super) structure_id: &'static Id<Structure>,
        /// The tile position of this ghost
        pub(crate) tile_pos: &'static TilePos,
        /// The inputs that must be added to construct this ghost
        pub(super) input_inventory: &'static InputInventory,
        /// The ghost's progress through construction
        pub(crate) crafting_state: &'static CraftingState,
        /// The signal emitter
        pub(super) emitter: &'static Emitter,
        /// The recipe that will be crafted when the structure is first built
        pub(super) active_recipe: &'static ActiveRecipe,
    }

    /// Detailed info about a given ghost.
    #[derive(Debug)]
    pub(crate) struct GhostDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: Id<Structure>,
        /// The inputs that must be added to construct this ghost
        pub(super) input_inventory: InputInventory,
        /// The ghost's progress through construction
        pub(super) crafting_state: CraftingState,
        /// The recipe that will be crafted when the structure is first built
        pub(super) active_recipe: ActiveRecipe,
    }

    impl GhostDetails {
        /// The pretty formatting for this type
        pub(crate) fn display(
            &self,
            item_manifest: &ItemManifest,
            structure_manifest: &StructureManifest,
            recipe_manifest: &RecipeManifest,
        ) -> String {
            let entity = self.entity;
            let structure_id = structure_manifest.name(self.structure_id);
            let tile_pos = &self.tile_pos;
            let crafting_state = &self.crafting_state;
            let recipe = self.active_recipe.display(recipe_manifest);
            let construction_materials = self.input_inventory.display(item_manifest);

            format!(
                "Entity: {entity:?}
Tile: {tile_pos}
Ghost structure type: {structure_id}
Recipe: {recipe}
Construction materials: {construction_materials}
{crafting_state}"
            )
        }
    }
}

/// Details for organisms
mod organism_details {
    use bevy::ecs::query::WorldQuery;

    use crate::{
        asset_management::manifest::{StructureManifest, UnitManifest},
        organisms::{energy::EnergyPool, lifecycle::Lifecycle, OrganismId},
    };

    /// Data needed to populate [`OrganismDetails`].
    #[derive(WorldQuery)]
    pub(super) struct OrganismDetailsQuery {
        /// The organism's current progress through its lifecycle
        pub(super) lifecycle: &'static Lifecycle,
        /// The current and max energy
        pub(super) energy_pool: &'static EnergyPool,
    }

    /// Detailed info about a given organism.
    #[derive(Debug)]
    pub(crate) struct OrganismDetails {
        /// The prototypical "base" bersion of this orgnaism
        pub(super) prototypical_form: OrganismId,
        /// The organism's current progress through its lifecycle
        pub(super) lifecycle: Lifecycle,
        /// The current and max energy
        pub(super) energy_pool: EnergyPool,
    }

    impl OrganismDetails {
        /// Pretty formatting for this type
        pub(crate) fn display(
            &self,
            structure_manifest: &StructureManifest,
            unit_manifest: &UnitManifest,
        ) -> String {
            let prototypical_form = self
                .prototypical_form
                .display(structure_manifest, unit_manifest);
            let lifecycle = self.lifecycle.display(structure_manifest, unit_manifest);

            let energy_pool = &self.energy_pool;

            format!(
                "Prototypical form: {prototypical_form}
Lifecycle: {lifecycle}
Energy: {energy_pool}"
            )
        }
    }
}

/// Details for structures
mod structure_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use super::organism_details::OrganismDetails;
    use crate::{
        asset_management::manifest::{
            Id, ItemManifest, Structure, StructureManifest, UnitManifest,
        },
        items::{inventory::Inventory, recipe::RecipeData},
        simulation::geometry::TilePos,
        structures::{
            construction::MarkedForDemolition,
            crafting::{
                ActiveRecipe, CraftingState, InputInventory, OutputInventory, StorageInventory,
                WorkersPresent,
            },
        },
    };

    /// Data needed to populate [`StructureDetails`].
    #[derive(WorldQuery)]
    pub(super) struct StructureDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of structure
        pub(super) structure_id: &'static Id<Structure>,
        /// The tile position of this structure
        pub(crate) tile_pos: &'static TilePos,
        /// Crafting-related components
        pub(super) crafting: Option<(
            &'static InputInventory,
            &'static OutputInventory,
            &'static ActiveRecipe,
            &'static WorkersPresent,
            &'static CraftingState,
        )>,
        /// If this structure stores things, its inventory.
        pub(super) storage_inventory: Option<&'static StorageInventory>,
        /// Is this structure marked for removal?
        pub(super) marked_for_removal: Option<&'static MarkedForDemolition>,
    }

    /// Detailed info about a given structure.
    #[derive(Debug)]
    pub(crate) struct StructureDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: Id<Structure>,
        /// If this structure is crafting something, the details about that.
        pub(crate) crafting_details: Option<CraftingDetails>,
        /// If this structure stores things, its inventory.
        pub(crate) storage_inventory: Option<StorageInventory>,
        /// Details about this organism, if it is one.
        pub(crate) maybe_organism_details: Option<OrganismDetails>,
        /// Is this structure slated for removal?
        pub(crate) marked_for_removal: bool,
    }

    impl StructureDetails {
        /// The pretty foramtting for this type
        pub(crate) fn display(
            &self,
            structure_manifest: &StructureManifest,
            unit_manifest: &UnitManifest,
            item_manifest: &ItemManifest,
        ) -> String {
            let entity = self.entity;
            let structure_id = structure_manifest.name(self.structure_id);
            let tile_pos = &self.tile_pos;

            let mut string = format!(
                "Entity: {entity:?}
Structure type: {structure_id}
Tile: {tile_pos}"
            );

            if self.marked_for_removal {
                string += "\nMarked for removal!";
            }

            if let Some(crafting) = &self.crafting_details {
                string += &format!("\n{}", crafting.display(item_manifest));
            }

            if let Some(storage) = &self.storage_inventory {
                string += &format!("\nStoring: {}", storage.display(item_manifest));
            }

            if let Some(organism) = &self.maybe_organism_details {
                string += &format!("\n{}", organism.display(structure_manifest, unit_manifest));
            };

            string
        }
    }

    /// The details about crafting processes.
    #[derive(Debug, Clone)]
    pub(crate) struct CraftingDetails {
        /// The inventory for the input items.
        pub(crate) input_inventory: Inventory,

        /// The inventory for the output items.
        pub(crate) output_inventory: Inventory,

        /// The recipe used, if any.
        pub(crate) recipe: Option<RecipeData>,

        /// The state of the ongoing crafting process.
        pub(crate) state: CraftingState,

        /// The number of workers that are presently working on this.
        pub(crate) workers_present: WorkersPresent,
    }

    impl CraftingDetails {
        /// The pretty formatting for this type.
        pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
            let input_inventory = self.input_inventory.display(item_manifest);
            let output_inventory = self.output_inventory.display(item_manifest);
            let crafting_state = &self.state;

            let recipe_string = match &self.recipe {
                Some(recipe) => recipe.display(item_manifest),
                None => "None".to_string(),
            };

            let workers_present = &self.workers_present;

            format!(
                "Recipe: {recipe_string}
Input: {input_inventory}
{crafting_state}
Workers present: {workers_present}
Output: {output_inventory}"
            )
        }
    }
}

/// Details for terrain
mod terrain_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::{
            Id, ItemManifest, StructureManifest, Terrain, TerrainManifest,
        },
        player_interaction::zoning::Zoning,
        signals::LocalSignals,
        simulation::geometry::{Height, TilePos},
    };

    /// Data needed to populate [`TerrainDetails`].
    #[derive(WorldQuery)]
    pub(super) struct TerrainDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The height of the tile
        pub(super) height: &'static Height,
        /// The type of terrain
        pub(super) terrain_id: &'static Id<Terrain>,
        /// The zoning applied to this terrain
        pub(super) zoning: &'static Zoning,
    }

    /// Detailed info about a given piece of terrain.
    #[derive(Debug)]
    pub(crate) struct TerrainDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of terrain
        pub(super) terrain_id: Id<Terrain>,
        /// The location of the tile
        pub(super) tile_pos: TilePos,
        /// The height of the tile
        pub(super) height: Height,
        /// The signals on this tile
        pub(super) signals: LocalSignals,
        /// The zoning of this tile
        pub(super) zoning: Zoning,
    }

    impl TerrainDetails {
        /// The pretty formatting for this type
        pub(crate) fn display(
            &self,
            terrain_manifest: &TerrainManifest,
            structure_manifest: &StructureManifest,
            item_manifest: &ItemManifest,
        ) -> String {
            let entity = self.entity;
            let terrain_type = terrain_manifest.name(self.terrain_id);
            let tile_pos = &self.tile_pos;
            let height = &self.height;
            let signals = self.signals.display(item_manifest, structure_manifest);
            let zoning = self.zoning.display(structure_manifest, terrain_manifest);

            format!(
                "Entity: {entity:?}
Terrain type: {terrain_type}
Tile: {tile_pos}
Height: {height}
Zoning: {zoning}
Signals:
{signals}"
            )
        }
    }
}

/// Details for units
mod unit_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::{Id, ItemManifest, StructureManifest, Unit, UnitManifest},
        simulation::geometry::TilePos,
        units::{
            actions::CurrentAction, goals::Goal, hunger::Diet, impatience::ImpatiencePool,
            item_interaction::UnitInventory,
        },
    };

    use super::organism_details::OrganismDetails;

    /// Data needed to populate [`UnitDetails`].
    #[derive(WorldQuery)]
    pub(super) struct UnitDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of unit
        pub(super) unit_id: &'static Id<Unit>,
        /// What does this unit eat?
        pub(super) diet: &'static Diet,
        /// The current location
        pub(super) tile_pos: &'static TilePos,
        /// What's being carried
        pub(super) held_item: &'static UnitInventory,
        /// What this unit is trying to achieve
        pub(super) goal: &'static Goal,
        /// What is currently being done
        pub(super) action: &'static CurrentAction,
        /// How frustrated the unit is
        pub(super) impatience_pool: &'static ImpatiencePool,
    }

    /// Detailed info about a given unit.
    #[derive(Debug)]
    pub(crate) struct UnitDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of unit
        pub(super) unit_id: Id<Unit>,
        /// What does this unit eat?
        pub(super) diet: Diet,
        /// The current location
        pub(super) tile_pos: TilePos,
        /// What's being carried
        pub(super) held_item: UnitInventory,
        /// What this unit is trying to achieve
        pub(super) goal: Goal,
        /// What is currently being done
        pub(super) action: CurrentAction,
        /// Details about this organism, if it is one.
        pub(crate) organism_details: OrganismDetails,
        /// How frustrated the unit is
        pub(super) impatience_pool: ImpatiencePool,
    }

    impl UnitDetails {
        /// The pretty formatting for this type.
        pub(crate) fn display(
            &self,
            unit_manifest: &UnitManifest,
            item_manifest: &ItemManifest,
            structure_manifest: &StructureManifest,
        ) -> String {
            let entity = self.entity;
            let unit_name = unit_manifest.name(self.unit_id);
            let diet = self.diet.display(item_manifest);
            let tile_pos = &self.tile_pos;
            let held_item = self.held_item.display(item_manifest);
            let goal = self.goal.display(item_manifest, structure_manifest);
            let action = &self.action.display(item_manifest);
            let impatience_pool = &self.impatience_pool;
            let organism_details = self
                .organism_details
                .display(structure_manifest, unit_manifest);

            format!(
                "Entity: {entity:?}
Unit type: {unit_name}
Tile: {tile_pos}
Diet: {diet}
Holding: {held_item}
Goal: {goal}
Action: {action}
Impatience: {impatience_pool}
{organism_details}"
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
