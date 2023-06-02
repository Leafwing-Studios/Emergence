//! The clipboard stores selected structures, to later be placed via zoning.

use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashMap};
use hexx::HexIterExt;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::Id,
    construction::{ghosts::Preview, terraform::TerraformingTool},
    crafting::recipe::ActiveRecipe,
    geometry::{Facing, Height, MapGeometry, VoxelPos},
    structures::structure_manifest::{Structure, StructureManifest},
};

use super::{picking::CursorPos, selection::CurrentSelection, InteractionSystem, PlayerAction};

/// Code and data for working with the clipboard
pub(super) struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tool>()
            // We're running this before we select tiles to deliberately introduce a one-frame delay,
            // ensuring that users need to double click to clear the clipboard as well.
            .add_system(clear_clipboard.before(InteractionSystem::SelectTiles))
            .add_system(
                copy_selection
                    .in_set(InteractionSystem::SetClipboard)
                    .after(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::SelectTiles),
            )
            .add_system(
                rotate_selection
                    .in_set(InteractionSystem::SetClipboard)
                    .after(copy_selection),
            );
    }
}

/// Stores the currently active tool that the player is using.
#[derive(Default, Resource, Debug)]
pub(crate) enum Tool {
    /// Terraform terrain.
    Terraform(TerraformingTool),
    /// A structure / structure to place
    Structures(HashMap<VoxelPos, ClipboardData>),
    /// No tool is selected.
    #[default]
    None,
}

impl Tool {
    /// Is the clipboard empty?
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Tool::None => true,
            Tool::Structures(map) => map.is_empty(),
            Tool::Terraform(_) => false,
        }
    }

    /// Sets the contents of the clipboard to a single structure (or clears it if [`None`] is provided).
    pub(crate) fn set_to_structure(&mut self, maybe_structure: Option<ClipboardData>) {
        *self = match maybe_structure {
            Some(clipboard_data) => Tool::Structures({
                let mut map = HashMap::new();
                map.insert(VoxelPos::default(), clipboard_data);
                map
            }),
            None => Tool::None,
        };
    }
}

/// The data copied via the clipboard for a single structure.
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ClipboardData {
    /// The identity of the structure.
    pub(crate) structure_id: Id<Structure>,
    /// The orientation of the structure.
    pub(crate) facing: Facing,
    /// The recipe that this structure makes, if any
    pub(crate) active_recipe: ActiveRecipe,
}

impl ClipboardData {
    /// Creates a new [`ClipboardData`] for the given `structure_id`.
    ///
    /// The starting properties are determined by the `structure_manifest`.
    /// This is intended to be used during world generation.
    #[must_use]
    pub(crate) fn generate_from_id(
        structure_id: Id<Structure>,
        structure_manifest: &StructureManifest,
    ) -> Self {
        Self {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        }
    }
}

impl Tool {
    /// Normalizes the positions of the items on the clipboard.
    ///
    /// Centers relative to the median selected tile position.
    /// Each axis is computed independently.
    fn normalize_positions(&mut self) {
        if let Tool::Structures(map) = self {
            let center_hex = map.keys().map(|voxel_pos| voxel_pos.hex).center();

            // FIXME: it's unclear if this height is correct
            let center = VoxelPos::new(center_hex, Height::ZERO);

            let mut new_map = HashMap::with_capacity(map.capacity());

            for (voxel_pos, id) in map.iter() {
                let new_tile_pos = *voxel_pos - center;
                // PERF: eh maybe we can safe a clone by using remove?
                new_map.insert(new_tile_pos, id.clone());
            }

            *map = new_map;
        }
    }

    /// Apply a tile-position shift to the items on the clipboard.
    ///
    /// Used to place items in the correct location relative to the cursor.
    pub(crate) fn offset_positions(&self, origin: VoxelPos) -> Vec<(VoxelPos, ClipboardData)> {
        if let Tool::Structures(map) = self {
            map.iter()
                .map(|(k, v)| ((*k + origin), v.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Rotates the contents of the clipboard around the origin.
    ///
    /// You must ensure that the contents are normalized first.
    fn rotate(&mut self, clockwise: bool) {
        if let Tool::Structures(map) = self {
            let mut new_map = HashMap::with_capacity(map.capacity());

            for (&original_pos, item) in map.iter_mut() {
                let new_pos = if clockwise {
                    item.facing.rotate_clockwise();
                    original_pos.hex.clockwise()
                } else {
                    item.facing.rotate_counterclockwise();
                    original_pos.hex.counter_clockwise()
                };

                new_map.insert(VoxelPos::new(new_pos, Height::ZERO), item.clone());
            }

            *map = new_map;
        }
    }
}

/// Clears the clipboard when the correct actions are pressed
fn clear_clipboard(mut tool: ResMut<Tool>, actions: Res<ActionState<PlayerAction>>) {
    if actions.just_pressed(PlayerAction::Deselect) {
        *tool = Tool::None;
    }
}

/// Data needed for [`copy_selection`] to populate [`ClipboardData`].
#[derive(WorldQuery)]
struct ClipboardQuery {
    /// The position of the structure
    voxel_pos: &'static VoxelPos,
    /// The type of the structure
    structure_id: &'static Id<Structure>,
    /// The direction the structure is facing
    facing: &'static Facing,
    /// The recipe that the structure is crafting, if any
    active_recipe: Option<&'static ActiveRecipe>,
}

impl From<ClipboardQueryItem<'_>> for ClipboardData {
    fn from(value: ClipboardQueryItem) -> ClipboardData {
        let active_recipe = match value.active_recipe {
            Some(recipe) => recipe.clone(),
            None => ActiveRecipe::default(),
        };

        ClipboardData {
            structure_id: *value.structure_id,
            facing: *value.facing,
            active_recipe,
        }
    }
}

/// Copies the selected structure(s) to the clipboard, to be placed later.
fn copy_selection(
    actions: Res<ActionState<PlayerAction>>,
    mut tool: ResMut<Tool>,
    cursor_pos: Res<CursorPos>,
    current_selection: Res<CurrentSelection>,
    structure_query: Query<ClipboardQuery, Without<Preview>>,
    map_geometry: Res<MapGeometry>,
) {
    if actions.just_pressed(PlayerAction::Copy) {
        // We want to replace our selection, rather than add to it
        let mut map = HashMap::new();

        match &*current_selection {
            CurrentSelection::Structure(entity) | CurrentSelection::GhostStructure(entity) => {
                let query_item = structure_query.get(*entity).unwrap();
                let voxel_pos = query_item.voxel_pos;
                let clipboard_data = query_item.into();
                map.insert(*voxel_pos, clipboard_data);
                *tool = Tool::Structures(map);
                tool.normalize_positions();
            }
            CurrentSelection::Terrain(selected_tiles) => {
                // If there is no selection, just grab whatever's under the cursor
                if selected_tiles.is_empty() {
                    if let Some(hovered_tile) = cursor_pos.maybe_tile_pos() {
                        if let Some(entity) = map_geometry.get_ghost_structure(hovered_tile) {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(VoxelPos::default(), clipboard_data);
                        } else if let Some(entity) = map_geometry.get_structure(hovered_tile) {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(VoxelPos::default(), clipboard_data);
                        }
                    }
                } else {
                    for &hex in selected_tiles.selection().iter() {
                        // TODO: this doesn't let us select multiple layers of structures effectively
                        let voxel_pos = map_geometry.on_top_of_terrain(hex);
                        if let Some(entity) = map_geometry.get_ghost_structure(voxel_pos) {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(VoxelPos::default(), clipboard_data);
                        } else if let Some(entity) = map_geometry.get_structure(voxel_pos) {
                            let clipboard_data = structure_query.get(entity).unwrap().into();
                            map.insert(VoxelPos::default(), clipboard_data);
                        }
                    }
                    *tool = Tool::Structures(map);
                    tool.normalize_positions();
                }
            }
            // Otherwise, just grab whatever's under the cursor
            CurrentSelection::None | CurrentSelection::Unit(_) => {
                if let Some(cursor_tile_pos) = cursor_pos.maybe_tile_pos() {
                    if let Some(structure_entity) = map_geometry.get_structure(cursor_tile_pos) {
                        let clipboard_data = structure_query.get(structure_entity).unwrap().into();
                        map.insert(VoxelPos::default(), clipboard_data);
                        *tool = Tool::Structures(map);
                    }
                }
            }
        }
    }
}

/// Rotates the contents of the clipboard based on player input
fn rotate_selection(actions: Res<ActionState<PlayerAction>>, mut clipboard: ResMut<Tool>) {
    if actions.just_pressed(PlayerAction::RotateClipboardLeft)
        && actions.just_pressed(PlayerAction::RotateClipboardRight)
    {
        return;
    }

    if actions.just_pressed(PlayerAction::RotateClipboardLeft) {
        clipboard.rotate(false);
    }

    if actions.just_pressed(PlayerAction::RotateClipboardRight) {
        clipboard.rotate(true);
    }
}
