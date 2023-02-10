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

/// The details about crafting processes.
#[derive(Debug, Clone)]
pub struct CraftingDetails {
    /// The inventory for the input items.
    pub input_inventory: Inventory,

    /// The inventory for the output items.
    pub output_inventory: Inventory,

    /// The recipe that's currently being crafted, if any.
    pub active_recipe: Option<RecipeId>,

    /// The state of the ongoing crafting process.
    pub state: CraftingState,

    /// The time remaining to finish crafting.
    pub timer: Timer,
}

/// Detailed info about a given entity.
#[derive(Debug, Clone)]
pub struct StructureDetails {
    /// The entity ID of the structure that this info is about.
    pub entity: Entity,
    /// The tile position of this organism.
    pub tile_pos: TilePos,
    /// The type of structure, e.g. plant or fungus.
    pub structure_id: StructureId,
    /// If this organism is crafting something, the details about that.
    pub crafting_details: Option<CraftingDetails>,
}

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default, Deref)]
pub(crate) struct SelectionDetails {
    pub(crate) structure: Option<StructureDetails>,
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
struct HoverDetailsQuery {
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
    mut hover_details: ResMut<SelectionDetails>,
    structure_query: Query<HoverDetailsQuery>,
    map_geometry: Res<MapGeometry>,
) {
    if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        hover_details.structure = None;

        if let Some(&structure_entity) = map_geometry.structure_index.get(&cursor_pos) {
            let structure_details = structure_query.get(structure_entity).unwrap();

            let crafting_details =
                if let Some((input, output, recipe, state, timer)) = structure_details.crafting {
                    Some(CraftingDetails {
                        input_inventory: input.inventory().clone(),
                        output_inventory: output.inventory().clone(),
                        active_recipe: recipe.recipe_id().clone(),
                        state: state.clone(),
                        timer: timer.timer().clone(),
                    })
                } else {
                    None
                };

            hover_details.structure = Some(StructureDetails {
                entity: structure_entity,
                tile_pos: cursor_pos,
                structure_id: structure_details.structure_id.clone(),
                crafting_details,
            });
        } else {
            hover_details.structure = None;
        }
    }
}
