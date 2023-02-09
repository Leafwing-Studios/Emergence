//! Detailed info about a given organism.

use bevy::prelude::*;

use crate::{
    items::{inventory::Inventory, recipe::RecipeId},
    player_interaction::cursor::CursorPos,
    simulation::geometry::MapGeometry,
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
    /// The entity ID of the organism that this info is about.
    pub entity: Entity,

    /// The type of structure, e.g. plant or fungus.
    pub structure_id: StructureId,

    /// If this organism is crafting something, the details about that.
    pub crafting_details: Option<CraftingDetails>,
}

/// Detailed info about the organism that is being hovered.
#[derive(Debug, Resource, Default, Deref)]
pub struct HoverDetails(Option<StructureDetails>);

/// Display detailed info on hover.
pub struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building DetailsPlugin...");

        app.init_resource::<HoverDetails>()
            // TODO: This should be done after the cursor system
            .add_system(hover_details);
    }
}

/// Get details about the hovered entity.
fn hover_details(
    cursor_pos: Res<CursorPos>,
    mut hover_details: ResMut<HoverDetails>,
    // TODO: use a WorldQuery type
    structure_query: Query<(
        Entity,
        &StructureId,
        Option<(
            &InputInventory,
            &OutputInventory,
            &ActiveRecipe,
            &CraftingState,
            &CraftTimer,
        )>,
    )>,
    map_geometry: Res<MapGeometry>,
) {
    if let Some(cursor_pos) = cursor_pos.maybe_tile_pos() {
        hover_details.0 = None;

        if let Some(&structure_entity) = map_geometry.structure_index.get(&cursor_pos) {
            let structure_details = structure_query.get(structure_entity).unwrap();

            let crafting_details =
                if let Some((input, output, recipe, state, timer)) = structure_details.2 {
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

            hover_details.0 = Some(StructureDetails {
                entity: structure_entity,
                structure_id: structure_details.1.clone(),
                crafting_details,
            });
        } else {
            hover_details.0 = None;
        }
    }
}
