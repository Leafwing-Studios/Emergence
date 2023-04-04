//! Making more units

use bevy::prelude::*;
use rand::prelude::IteratorRandom;
use rand::thread_rng;

use crate::{
    asset_management::manifest::Id,
    crafting::{ActiveRecipe, CraftingState},
    simulation::geometry::{MapGeometry, TilePos},
};

use super::{unit_assets::UnitHandles, unit_manifest::UnitManifest, UnitBundle};

/// Spawn ants when eggs have hatched
pub(super) fn hatch_ant_eggs(
    structure_query: Query<(&TilePos, &CraftingState, &ActiveRecipe)>,
    map_geometry: Res<MapGeometry>,
    unit_handles: Res<UnitHandles>,
    unit_manifest: Res<UnitManifest>,
    mut commands: Commands,
) {
    let rng = &mut thread_rng();

    // PERF: I don't like the linear time polling here. This really feels like it should be push-based with one-shot system callbacks on the recipe.
    for (tile_pos, crafting_state, active_recipe) in structure_query.iter() {
        if let Some(recipe_id) = active_recipe.recipe_id() {
            // TODO: This should be generalized to be driven by the recipe manifest.
            if *recipe_id == Id::from_name("hatch_ants".to_string())
                && matches!(crafting_state, CraftingState::RecipeComplete)
            {
                let empty_neighbors = tile_pos.empty_neighbors(&map_geometry);
                if let Some(pos_to_spawn) = empty_neighbors.into_iter().choose(rng) {
                    commands.spawn(UnitBundle::new(
                        Id::from_name("ant".to_string()),
                        pos_to_spawn,
                        unit_manifest.get(Id::from_name("ant".to_string())).clone(),
                        &unit_handles,
                        &map_geometry,
                    ));
                }
            }
        }
    }
}
