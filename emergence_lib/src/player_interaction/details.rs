//! Detailed info about a given organism.

use bevy::prelude::*;

use self::structure::*;
use self::unit::*;

use crate::{
    player_interaction::{cursor::CursorPos, InteractionSystem},
    simulation::geometry::MapGeometry,
};

use super::tile_selection::SelectedTiles;

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default)]
pub(crate) enum SelectionDetails {
    /// A structure is selected
    Structure(StructureDetails),
    /// A unit is selected
    Unit(UnitDetails),
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

mod structure {
    use bevy::{
        ecs::{prelude::*, query::WorldQuery},
        time::Timer,
    };

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
        /// The tile position of this organism.
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: StructureId,
        /// If this organism is crafting something, the details about that.
        pub(crate) crafting_details: Option<CraftingDetails>,
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
}

mod unit {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        organisms::units::{
            behavior::{CurrentAction, Goal},
            item_interaction::HeldItem,
            UnitId,
        },
        simulation::geometry::TilePos,
    };

    /// Data needed to populate [`StructureDetails`].
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
}
