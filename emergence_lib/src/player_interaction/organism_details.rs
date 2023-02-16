//! Detailed info about a given organism.

use bevy::{ecs::query::WorldQuery, prelude::*};

use crate::{
    items::{inventory::Inventory, recipe::RecipeId},
    player_interaction::{cursor::CursorPos, InteractionSystem},
    simulation::geometry::{MapGeometry, TilePos},
    structures::{
        crafting::{ActiveRecipe, CraftTimer, CraftingState, InputInventory, OutputInventory},
        StructureId,
    },
};

use super::tile_selection::SelectedTiles;

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

/// Detailed info about a given entity.
#[derive(Debug, Clone)]
pub(crate) struct StructureDetails {
    /// The tile position of this organism.
    pub(crate) tile_pos: TilePos,
    /// The type of structure, e.g. plant or fungus.
    pub(crate) structure_id: StructureId,
    /// If this organism is crafting something, the details about that.
    pub(crate) crafting_details: Option<CraftingDetails>,
}

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default)]
pub(crate) enum SelectionDetails {
    /// A structure is selected
    Structure(StructureDetails),
    /// Nothing is selected
    #[default]
    None,
}

impl SelectionDetails {
    /// Is this [`SelectionDetails::None`]?
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, SelectionDetails::None)
    }
}

/// Display detailed info on hover.
pub(super) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building DetailsPlugin...");

        app.init_resource::<SelectionDetails>().add_system(
            hover_details
                .label(InteractionSystem::HoverDetails)
                .after(InteractionSystem::SelectTiles),
        );
    }
}

/// Data needed to populate [`StructureDetails`].
#[derive(WorldQuery)]
struct StructureDetailsQuery {
    /// The type of structure
    structure_id: &'static StructureId,
    /// The location
    tile_pos: &'static TilePos,
    /// Crafting-related components
    crafting: Option<(
        &'static InputInventory,
        &'static OutputInventory,
        &'static ActiveRecipe,
        &'static CraftingState,
        &'static CraftTimer,
    )>,
}

/// Get details about the hovered entity.
fn hover_details(
    cursor_pos: Res<CursorPos>,
    selected_tiles: Res<SelectedTiles>,
    mut selection_details: ResMut<SelectionDetails>,
    structure_query: Query<StructureDetailsQuery>,
    map_geometry: Res<MapGeometry>,
) {
    // If only one tile is selected, use that.
    let tile_pos = if selected_tiles.selection().len() == 1 {
        *selected_tiles.selection().iter().next().unwrap()
    // Otherwise use the cursor
    } else if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        cursor_pos
    // If the cursor isn't over a tile, just return early
    } else {
        return;
    };

    *selection_details = SelectionDetails::None;

    if let Some(&structure_entity) = map_geometry.structure_index.get(&tile_pos) {
        let structure_details = structure_query.get(structure_entity).unwrap();

        let crafting_details =
            if let Some((input, output, recipe, state, timer)) = structure_details.crafting {
                Some(CraftingDetails {
                    input_inventory: input.inventory.clone(),
                    output_inventory: output.inventory.clone(),
                    active_recipe: recipe.recipe_id().clone(),
                    state: state.clone(),
                    timer: timer.timer().clone(),
                })
            } else {
                None
            };

        *selection_details = SelectionDetails::Structure(StructureDetails {
            tile_pos,
            structure_id: structure_details.structure_id.clone(),
            crafting_details,
        });
    } else {
        *selection_details = SelectionDetails::None;
    }
}
