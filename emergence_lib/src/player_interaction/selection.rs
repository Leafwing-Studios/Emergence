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
    selection: HashSet<(Entity, TilePos)>,
}

impl SelectedTiles {
    /// Selects a single tile
    fn add_tile(&mut self, tile_entity: Entity, tile_pos: TilePos) {
        self.selection.insert((tile_entity, tile_pos));
    }

    /// Deselects a single tile
    fn remove_tile(&mut self, tile_entity: Entity, tile_pos: TilePos) {
        self.selection.remove(&(tile_entity, tile_pos));
    }

    /// Selects a hexagon of tiles.
    fn select_hexagon(
        &mut self,
        center: TilePos,
        radius: u32,
        map_geometry: &MapGeometry,
        select: bool,
    ) {
        let hex_coord = hexagon(center.hex, radius);

        for hex in hex_coord {
            let target_pos = TilePos { hex };
            // Selection may have overflowed map
            if let Some(target_entity) = map_geometry.terrain_index.get(&target_pos) {
                match select {
                    true => self.add_tile(*target_entity, target_pos),
                    false => self.remove_tile(*target_entity, target_pos),
                }
            }
        }
    }

    /// The current set of selected tiles
    fn selection(&self) -> &HashSet<(Entity, TilePos)> {
        &self.selection
    }

    /// Clears the set of selected tiles.
    fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Are any tiles selected?
    fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }

    /// Is the given tile in the selection?
    fn contains_tile(&self, tile_entity: Entity, tile_pos: TilePos) -> bool {
        self.selection.contains(&(tile_entity, tile_pos))
    }
}

/// All tile, structure and unit selection logic and graphics
pub(super) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTiles>()
            .init_resource::<ActionState<SelectionAction>>()
            .init_resource::<Clipboard>()
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

/// Integrates user input into tile selection actions to let other systems handle what happens to a selected tile
#[allow(clippy::too_many_arguments)]
fn select_tiles(
    cursor: Res<CursorPos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<SelectionAction>>,
    mut selection_start: Local<Option<TilePos>>,
    mut initial_selection: Local<Option<SelectedTiles>>,
    mut previous_radius: Local<u32>,
    map_geometry: Res<MapGeometry>,
) {
    if let (Some(cursor_entity), Some(cursor_tile)) =
        (cursor.maybe_entity(), cursor.maybe_tile_pos())
    {
        let multiple = actions.pressed(SelectionAction::Multiple);
        let area = actions.pressed(SelectionAction::Area);
        let select = actions.pressed(SelectionAction::Select);
        let deselect = actions.pressed(SelectionAction::Deselect);

        // Cache the previous state to make area selection reversible
        if area & !multiple & initial_selection.is_none() {
            *selection_start = Some(cursor_tile);
            *initial_selection = Some(selected_tiles.clone());
        // Unless we're in the middle of an area selection, clear the cache
        } else if !(area & !multiple) | (!select & !deselect) {
            *selection_start = None;
            *initial_selection = None;
        }

        // Don't attempt to handle conflicting inputs.
        if select & deselect {
            return;
        }

        // This system has no work to do if we are not atempting to select or deselect anything
        if !select & !deselect {
            return;
        }

        // Selection logic
        if select {
            match (multiple, area) {
                // Simple select
                (false, false) => {
                    selected_tiles.clear_selection();
                    selected_tiles.add_tile(cursor_entity, cursor_tile)
                }
                // Multiple select
                (true, false) => selected_tiles.add_tile(cursor_entity, cursor_tile),
                // Area select
                (false, true) => {
                    let radius = cursor_tile.unsigned_distance_to(selection_start.unwrap().hex);
                    *previous_radius = radius;

                    // We need to be able to expand and shrink the selection reversibly
                    // so we need a snapshot of the state before this action took place.
                    *selected_tiles = initial_selection.as_ref().unwrap().clone();
                    selected_tiles.select_hexagon(
                        selection_start.unwrap(),
                        radius,
                        map_geometry.as_ref(),
                        true,
                    );
                }
                // Multiple area select
                (true, true) => selected_tiles.select_hexagon(
                    cursor_tile,
                    *previous_radius,
                    &map_geometry,
                    true,
                ),
            }
        }

        // Deselection logic
        if deselect {
            match (multiple, area) {
                // Simple deselect
                (false, false) => selected_tiles.clear_selection(),
                // Multiple deselect
                (true, false) => selected_tiles.remove_tile(cursor_entity, cursor_tile),
                // Area deselect
                (false, true) => {
                    let radius = cursor_tile.unsigned_distance_to(selection_start.unwrap().hex);
                    *previous_radius = radius;

                    // We need to be able to expand and shrink the selection reversibly
                    // so we need a snapshot of the state before this action took place.
                    *selected_tiles = initial_selection.as_ref().unwrap().clone();
                    selected_tiles.select_hexagon(
                        selection_start.unwrap(),
                        radius,
                        map_geometry.as_ref(),
                        false,
                    );
                }
                // Multiple area deselect
                (true, true) => selected_tiles.select_hexagon(
                    cursor_tile,
                    *previous_radius,
                    &map_geometry,
                    false,
                ),
            }
        }
    }
}

/// Shows which tiles are being hovered and selected.
fn display_tile_interactions(
    selected_tiles: Res<SelectedTiles>,
    mut terrain_query: Query<(Entity, &mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    cursor: Res<CursorPos>,
    materials: Res<TileHandles>,
) {
    if selected_tiles.is_changed() || cursor.is_changed() {
        let selection = selected_tiles.selection();
        // PERF: We should probably avoid a linear scan over all tiles here
        for (terrain_entity, mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let maybe_new_handle = if selection.contains(&(terrain_entity, tile_pos)) {
                if cursor.maybe_tile_pos() == Some(tile_pos) {
                    materials
                        .interaction_materials
                        .get(&ObjectInteraction::HoveredAndSelected)
                } else {
                    materials
                        .interaction_materials
                        .get(&ObjectInteraction::Selected)
                }
            } else {
                // This is somewhat clearer by comparison to the above branch when uncollapsed
                #[allow(clippy::collapsible_if)]
                if cursor.maybe_tile_pos() == Some(tile_pos) {
                    materials
                        .interaction_materials
                        .get(&ObjectInteraction::Hovered)
                } else {
                    materials.terrain_materials.get(terrain)
                }
            };

            *material = maybe_new_handle.unwrap().clone_weak();
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
                for (_terrain_entity, terrain_tile_pos) in selected_tiles.selection().iter() {
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
                    // PERF: this is kind of a mess; we can probably improve this through a smarter SelectedStructure type
                    let terrain_entity = map_geometry.terrain_index.get(tile_pos).unwrap();

                    if selected_tiles.contains_tile(*terrain_entity, *tile_pos) {
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
                    for (_terrain_entity, tile_pos) in selected_tiles.selection().iter() {
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
}
