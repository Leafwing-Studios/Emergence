//! Selecting tiles to be built on, inspected or modified

use crate as emergence_lib;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use emergence_macros::IterableEnum;

use hexx::{shapes::hexagon, Hex};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::{InputKind, Modifier},
    Actionlike,
};
use petitset::PetitSet;

use crate::{
    asset_management::TileHandles,
    simulation::geometry::{MapGeometry, TilePos},
    structures::{StructureBundle, StructureId},
    terrain::Terrain,
};

use super::{cursor::CursorPos, InteractionSystem};

/// Actions that can be used to select tiles.
///
/// If a tile is not selected, it will be added to the selection.
/// If it is already selected, it will be removed from the selection.
#[derive(Actionlike, Clone, Debug)]
pub enum SelectionAction {
    /// Selects a tile or group of tiles.
    Select,
    /// Deselects a tile or group of tiles.
    Deselect,
    /// Modifies the selection / deselection to be sequential.
    Multiple,
    /// Modifies the selection to cover a hexagonal area.
    Area,
    /// Modifies the selection to cover a line between the start and end of the selection.
    Line,
    /// Selects the structure on the tile under the player's cursor.
    ///
    /// If there is no structure there, the player's selection is cleared.
    Pipette,
    /// Sets the zoning of all currently selected tiles to the currently selected structure.
    ///
    /// If no structure is selected, any zoning will be removed.
    Zone,
}

impl SelectionAction {
    /// The default key bindings
    pub(super) fn default_input_map() -> InputMap<SelectionAction> {
        let mut control_shift_left_click = PetitSet::<InputKind, 8>::new();
        control_shift_left_click.insert(Modifier::Control.into());
        control_shift_left_click.insert(Modifier::Shift.into());
        control_shift_left_click.insert(MouseButton::Left.into());

        InputMap::default()
            .insert(MouseButton::Left, SelectionAction::Select)
            .insert(MouseButton::Right, SelectionAction::Deselect)
            .insert(Modifier::Shift, SelectionAction::Multiple)
            .insert(Modifier::Control, SelectionAction::Area)
            .insert(Modifier::Alt, SelectionAction::Line)
            .insert(KeyCode::Q, SelectionAction::Pipette)
            .insert(KeyCode::Space, SelectionAction::Zone)
            .build()
    }
}

/// How a given object is being interacted with by the player.
#[derive(PartialEq, Eq, Hash, Clone, Debug, IterableEnum)]
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
}

impl ObjectInteraction {
    /// The material used by objects that are being interacted with.
    pub(crate) fn material(&self) -> StandardMaterial {
        let base_color = match self {
            ObjectInteraction::Selected => Color::DARK_GREEN,
            ObjectInteraction::Hovered => Color::YELLOW,
            ObjectInteraction::HoveredAndSelected => Color::YELLOW_GREEN,
        };

        StandardMaterial {
            base_color,
            ..Default::default()
        }
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default, Clone)]
pub struct SelectedTiles {
    /// Actively selected tiles
    selected: HashSet<TilePos>,
    /// Tiles that are hovered over
    hovered: HashSet<TilePos>,
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

    /// Selects a hexagon of tiles.
    fn select_hexagon(&mut self, center: TilePos, radius: u32, select: bool) {
        let hex_coord = hexagon(center.hex, radius);

        for hex in hex_coord {
            let target_pos = TilePos { hex };
            // Selection may have overflowed map
            match select {
                true => self.add_tile(target_pos),
                false => self.remove_tile(target_pos),
            }
        }
    }

    /// Clears the set of selected tiles.
    fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Are any tiles selected?
    fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Is the given tile in the selection?
    fn contains_tile(&self, tile_pos: TilePos) -> bool {
        self.selected.contains(&tile_pos)
    }
}

/// All tile, structure and unit selection logic and graphics
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTiles>()
            .init_resource::<ActionState<SelectionAction>>()
            .init_resource::<Clipboard>()
            .init_resource::<AreaSelection>()
            .init_resource::<LineSelection>()
            .insert_resource(SelectionAction::default_input_map())
            .add_plugin(InputManagerPlugin::<SelectionAction>::default())
            .add_system(
                select_tiles
                    .label(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            )
            .add_system(copy_selection.after(InteractionSystem::SelectTiles))
            .add_system(
                apply_zoning
                    .after(InteractionSystem::SelectTiles)
                    .after(copy_selection),
            )
            .add_system(display_tile_interactions.after(InteractionSystem::SelectTiles));
    }
}

/// The state needed by [`SelectionAction::Area`].
#[derive(Resource)]
struct AreaSelection {
    /// The central tile, where the area selection began.
    center: Option<TilePos>,
    /// The radius of the selection.
    radius: u32,
    /// The tiles selected at the start of this action.
    initial_selection: Option<SelectedTiles>,
}

impl AreaSelection {
    /// Set things up to start a line selection action.
    fn begin(&mut self, selected_tiles: &SelectedTiles, cursor_pos: TilePos) {
        self.center = Some(cursor_pos);
        self.initial_selection = Some(selected_tiles.clone());
    }

    /// Clean things up to conclude a line selection action.
    fn finish(&mut self) {
        self.center = None;
        self.initial_selection = None;
    }
}

impl Default for AreaSelection {
    fn default() -> Self {
        AreaSelection {
            center: None,
            radius: 1,
            initial_selection: None,
        }
    }
}

/// The state needed by [`SelectionAction::Line`].
#[derive(Resource, Default)]
struct LineSelection {
    /// The starting tile, where the line selection began.
    start: Option<TilePos>,
    /// The tiles selected at the start of this action.
    initial_selection: Option<SelectedTiles>,
}

impl LineSelection {
    /// Set things up to start a line selection action.
    fn begin(&mut self, selected_tiles: &SelectedTiles, cursor_pos: TilePos) {
        self.start = Some(cursor_pos);
        self.initial_selection = Some(selected_tiles.clone());
    }

    /// Clean things up to conclude a line selection action.
    fn finish(&mut self) {
        self.start = None;
        self.initial_selection = None;
    }

    /// Computes the set of hexagons between `self.start` and `end`, with a thickness determnind by `radius`.
    fn draw_line(&self, end: TilePos, radius: u32) -> HashSet<TilePos> {
        let start = self.start.unwrap();
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
}

/// Integrates user input into tile selection actions to let other systems handle what happens to a selected tile
#[allow(clippy::too_many_arguments)]
fn select_tiles(
    cursor: Res<CursorPos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<SelectionAction>>,
    mut area_selection: ResMut<AreaSelection>,
    mut line_selection: ResMut<LineSelection>,
) {
    if let Some(cursor_pos) = cursor.maybe_tile_pos() {
        let select = actions.pressed(SelectionAction::Select);
        let deselect = actions.pressed(SelectionAction::Deselect);

        let multiple = actions.pressed(SelectionAction::Multiple);
        let area = actions.pressed(SelectionAction::Area);
        let line = actions.pressed(SelectionAction::Line);
        let simple_area = area & !multiple & !line;
        let simple_deselect = deselect & !area & !multiple & !line;

        // Cache the starting state to make selections reversible

        if simple_area & area_selection.initial_selection.is_none() {
            area_selection.begin(&selected_tiles, cursor_pos);
        }

        if line & line_selection.initial_selection.is_none() {
            line_selection.begin(&selected_tiles, cursor_pos);
        }

        // Clean up state from area and line selections
        if !simple_area {
            area_selection.finish();
        }

        if !line {
            line_selection.finish();
        }

        // Compute the center and radius
        let (center, radius) = if area {
            let center = if !simple_area {
                cursor_pos
            } else {
                area_selection.center.unwrap()
            };

            if simple_area {
                area_selection.radius = cursor_pos.unsigned_distance_to(center.hex);
            }

            (center, area_selection.radius)
        } else {
            (cursor_pos, 0)
        };

        // Record which tiles should have the "hovered" effect
        selected_tiles.hovered.clear();
        if simple_area {
            selected_tiles.hovered.insert(center);
            let ring = center.hex.ring(radius);
            for hex in ring {
                selected_tiles.hovered.insert(TilePos { hex });
            }
        } else if line {
            let line_hexes = line_selection.draw_line(cursor_pos, radius);
            selected_tiles.hovered.extend(line_hexes);
        } else {
            selected_tiles.hovered.insert(cursor_pos);
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
            if actions.just_released(SelectionAction::Select) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                selected_tiles.selected.extend(line_hexes);
            } else if actions.just_released(SelectionAction::Deselect) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                for tile_pos in line_hexes {
                    selected_tiles.selected.remove(&tile_pos);
                }
            }
        } else {
            if select {
                selected_tiles.select_hexagon(center, radius, true);
            }

            if deselect {
                selected_tiles.select_hexagon(center, radius, false);
            }
        }
    }
}

/// Shows which tiles are being hovered and selected.
fn display_tile_interactions(
    selected_tiles: Res<SelectedTiles>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TileHandles>,
) {
    if selected_tiles.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = selected_tiles.hovered.contains(&tile_pos);
            let selected = selected_tiles.selected.contains(&tile_pos);

            *material = materials.get_material(terrain, hovered, selected);
        }
    }
}

/// Stores a selection to copy and paste.
#[derive(Default, Resource, Debug, Deref, DerefMut)]
struct Clipboard {
    /// The internal map of structures.
    contents: HashMap<TilePos, StructureId>,
}

impl Clipboard {
    /// Normalizes the positions of the items on the clipboard.
    ///
    /// Centers relative to the median selected tile position.
    /// Each axis is computed independently.
    fn normalize_positions(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut x_vec = Vec::from_iter(self.keys().map(|tile_pos| tile_pos.x));
        let mut y_vec = Vec::from_iter(self.keys().map(|tile_pos| tile_pos.y));

        x_vec.sort_unstable();
        y_vec.sort_unstable();

        let mid = self.len() / 2;
        let center = TilePos {
            hex: Hex {
                x: x_vec[mid],
                y: y_vec[mid],
            },
        };

        let mut new_map = HashMap::with_capacity(self.capacity());

        for (tile_pos, id) in self.iter() {
            let new_tile_pos = *tile_pos - center;
            // PERF: eh maybe we can safe a clone by using remove?
            new_map.insert(new_tile_pos, id.clone());
        }

        self.contents = new_map;
    }

    /// Apply a tile-position shift to the items on the clipboard.
    ///
    /// Used to place items in the correct location relative to the cursor.
    fn offset_positions(&self, origin: TilePos) -> Vec<(TilePos, StructureId)> {
        self.iter()
            .map(|(k, v)| ((*k + origin), v.clone()))
            .collect()
    }
}

// PERF: this pair of copy-paste systems should use an index of where the structures are
/// Copies the selected structure(s) to the clipboard, to be placed later.
///
/// This system also handles the "pipette" functionality.
fn copy_selection(
    cursor: Res<CursorPos>,
    actions: Res<ActionState<SelectionAction>>,
    mut clipboard: ResMut<Clipboard>,
    selected_tiles: Res<SelectedTiles>,
    structure_query: Query<(&StructureId, &TilePos)>,
) {
    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        if actions.just_pressed(SelectionAction::Pipette) {
            // We want to replace our selection, rather than add to it
            clipboard.clear();

            // If there is no selection, just grab whatever's under the cursor
            if selected_tiles.is_empty() {
                for (structure_id, structure_tile_pos) in structure_query.iter() {
                    if cursor_tile_pos == *structure_tile_pos {
                        clipboard.insert(TilePos::default(), structure_id.clone());
                        return;
                    }
                }
            } else {
                for terrain_tile_pos in selected_tiles.selected.iter() {
                    // PERF: lol quadratic...
                    for (structure_id, structure_tile_pos) in structure_query.iter() {
                        if terrain_tile_pos == structure_tile_pos {
                            clipboard.insert(*structure_tile_pos, structure_id.clone());
                        }
                    }
                }
            }

            clipboard.normalize_positions();
        }
    }
}

// PERF: this pair of copy-paste systems should use an index of where the structures are
/// Applies zoning to an area, causing structures to be created (or removed) there.
///
/// This system also handles the "paste" functionality.
fn apply_zoning(
    cursor: Res<CursorPos>,
    actions: Res<ActionState<SelectionAction>>,
    clipboard: Res<Clipboard>,
    structure_query: Query<(Entity, &TilePos), With<StructureId>>,
    map_geometry: Res<MapGeometry>,
    selected_tiles: Res<SelectedTiles>,
    mut commands: Commands,
) {
    if let Some(cursor_tile_pos) = cursor.maybe_tile_pos() {
        if actions.pressed(SelectionAction::Zone) {
            // Clear zoning
            if clipboard.is_empty() {
                // PERF: this needs to use an index, rather than a linear time search
                for (structure_entity, tile_pos) in structure_query.iter() {
                    if selected_tiles.contains_tile(*tile_pos) {
                        commands.entity(structure_entity).despawn();
                    }
                }
            // Zone using the single selected structure
            } else if clipboard.len() == 1 {
                let structure_id = clipboard.values().next().unwrap();

                if selected_tiles.is_empty() {
                    if map_geometry.height_index.contains_key(&cursor_tile_pos) {
                        // FIXME: this should use a dedicated command to get all the details right
                        commands.spawn(StructureBundle::new(structure_id.clone(), cursor_tile_pos));
                    }
                } else {
                    for tile_pos in selected_tiles.selected.iter() {
                        if map_geometry.height_index.contains_key(tile_pos) {
                            // FIXME: this should use a dedicated command to get all the details right
                            commands.spawn(StructureBundle::new(structure_id.clone(), *tile_pos));
                        }
                    }
                }
            // Paste the selection
            } else {
                for (tile_pos, structure_id) in clipboard.offset_positions(cursor_tile_pos) {
                    if map_geometry.height_index.contains_key(&tile_pos) {
                        // FIXME: this should use a dedicated command to get all the details right
                        commands.spawn(StructureBundle::new(structure_id.clone(), tile_pos));
                    }
                }
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
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(tile_pos);
        // Intentionally doubled
        selected_tiles.add_tile(tile_pos);
        selected_tiles.add_tile(tile_pos);
        selected_tiles.add_tile(tile_pos);

        assert_eq!(selected_tiles.selected.len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(tile_pos);
        selected_tiles.add_tile(tile_pos);
        selected_tiles.add_tile(tile_pos);

        assert_eq!(selected_tiles.selected.len(), 3);
        selected_tiles.clear_selection();
        assert_eq!(selected_tiles.selected.len(), 0);
    }
}
