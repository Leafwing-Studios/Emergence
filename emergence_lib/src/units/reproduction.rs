//! Making more units

use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;
use rand::prelude::IteratorRandom;
use rand::thread_rng;

use crate::{
    asset_management::units::UnitHandles,
    items::{recipe::RecipeId, ItemId},
    organisms::energy::{Energy, EnergyPool},
    simulation::geometry::{MapGeometry, TilePos},
    structures::crafting::{ActiveRecipe, CraftingState},
};

use super::{hunger::Diet, UnitBundle, UnitId};

/// Spawn ants when eggs have hatched
pub(super) fn hatch_ant_eggs(
    structure_query: Query<(&TilePos, &CraftingState, &ActiveRecipe)>,
    map_geometry: Res<MapGeometry>,
    unit_handles: Res<UnitHandles>,
    mut commands: Commands,
) {
    let rng = &mut thread_rng();

    // PERF: I don't like the linear time polling here. This really feels like it should be push-based with one-shot system callbacks on the recipe.
    for (tile_pos, crafting_state, active_recipe) in structure_query.iter() {
        if let Some(recipe_id) = active_recipe.recipe_id() {
            if *recipe_id == RecipeId::hatch_ants()
                && matches!(crafting_state, CraftingState::RecipeComplete)
            {
                let empty_neighbors = tile_pos.empty_neighbors(&map_geometry);
                if let Some(pos_to_spawn) = empty_neighbors.into_iter().choose(rng) {
                    // TODO: use a unit manifest instead
                    commands.spawn(UnitBundle::new(
                        UnitId::ant(),
                        pos_to_spawn,
                        EnergyPool::new_full(Energy(100.), Energy(-1.)),
                        Diet::new(ItemId::leuco_chunk(), Energy(50.)),
                        &unit_handles,
                        &map_geometry,
                    ));
                }
            }
        }
    }
}
