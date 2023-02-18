//! Detailed info about a given organism.

use bevy::prelude::*;

use self::structure::*;
use self::terrain::*;
use self::unit::*;

use crate::player_interaction::{cursor::CursorPos, InteractionSystem};
use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

/// Display detailed info on hover.
pub(super) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building DetailsPlugin...");

        app.init_resource::<SelectionType>()
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

/// The game entity currently selected for inspection.
#[derive(Resource, Debug, Default)]
enum SelectionType {
    /// A structure is selected
    Structure(Entity),
    /// A tile is selected
    Terrain(TilePos),
    /// A unit is selected
    Unit(Entity),
    /// Nothing is selected
    #[default]
    None,
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

/// Determine what should be selected
fn set_selection(mut selection_type: ResMut<SelectionType>, cursor_pos: Res<CursorPos>) {
    // TODO: use a fancier, more intuitive / controllable strategy here
    *selection_type = if let Some(unit_entity) = cursor_pos.maybe_unit() {
        SelectionType::Unit(unit_entity)
    } else if let Some(structure_entity) = cursor_pos.maybe_structure() {
        SelectionType::Structure(structure_entity)
    } else if let Some(tile_pos) = cursor_pos.maybe_tile_pos() {
        SelectionType::Terrain(tile_pos)
    } else {
        SelectionType::None
    }
}

/// Get details about the hovered entity.
fn get_details(
    selection_type: Res<SelectionType>,
    mut selection_details: ResMut<SelectionDetails>,
    terrain_query: Query<TerrainDetailsQuery>,
    unit_query: Query<UnitDetailsQuery>,
    structure_query: Query<StructureDetailsQuery>,
    map_geometry: Res<MapGeometry>,
) {
    *selection_details = match *selection_type {
        SelectionType::Terrain(tile_pos) => {
            let terrain_entity = map_geometry.terrain_index.get(&tile_pos).unwrap();
            let terrain_data = terrain_query.get(*terrain_entity).unwrap();

            SelectionDetails::Terrain(TerrainDetails {
                terrain_type: *terrain_data.terrain_type,
                tile_pos,
            })
        }
        SelectionType::Unit(unit_entity) => {
            let unit_query_item = unit_query.get(unit_entity).unwrap();
            SelectionDetails::Unit(UnitDetails {
                unit_id: *unit_query_item.unit_id,
                tile_pos: *unit_query_item.tile_pos,
                held_item: unit_query_item.held_item.clone(),
                goal: unit_query_item.goal.clone(),
                action: unit_query_item.action.clone(),
            })
        }
        SelectionType::Structure(structure_entity) => {
            let structure_query_item = structure_query.get(structure_entity).unwrap();

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
        SelectionType::None => SelectionDetails::None,
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
        organisms::units::{
            behavior::{CurrentAction, Goal},
            item_interaction::HeldItem,
            UnitId,
        },
        simulation::geometry::TilePos,
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
