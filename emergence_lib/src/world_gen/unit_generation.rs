//! Initializes organisms in the world.

use crate::asset_management::manifest::Id;
use crate::crafting::inventories::{CraftingState, InputInventory, OutputInventory};
use crate::crafting::recipe::{ActiveRecipe, RecipeManifest};
use crate::geometry::MapGeometry;
use crate::organisms::energy::EnergyPool;
use crate::simulation::rng::GlobalRng;
use crate::structures::structure_manifest::Structure;
use crate::units::unit_assets::UnitHandles;
use crate::units::unit_manifest::UnitManifest;
use crate::units::UnitBundle;

use bevy::prelude::*;
use rand::Rng;

use super::GenerationConfig;

/// Create starting units according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
pub(super) fn generate_units(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    maybe_unit_handles: Option<Res<UnitHandles>>,
    unit_manifest: Res<UnitManifest>,
    map_geometry: Res<MapGeometry>,
    mut rng: ResMut<GlobalRng>,
) {
    info!("Generating units...");

    // Collect out so we can mutate the height map to flatten the terrain while in the loop
    for voxel_pos in map_geometry.walkable_voxels() {
        for (&unit_id, &chance) in &config.unit_chances {
            if rng.gen::<f32>() < chance {
                let unit_bundle = if let Some(ref unit_handles) = maybe_unit_handles {
                    UnitBundle::randomized(
                        unit_id,
                        voxel_pos,
                        unit_manifest.get(unit_id).clone(),
                        unit_handles,
                        rng.get_mut(),
                    )
                } else {
                    UnitBundle::testing(
                        unit_id,
                        voxel_pos,
                        unit_manifest.get(unit_id).clone(),
                        rng.get_mut(),
                    )
                };

                commands.spawn(unit_bundle);
            }
        }
    }
}

/// Sets all the starting organisms to a random state to avoid strange synchronization issues.
pub(super) fn randomize_starting_organisms(
    // Energy pools for structures are randomized upon creation
    mut energy_pool_query: Query<&mut EnergyPool, Without<Id<Structure>>>,
    mut input_inventory_query: Query<&mut InputInventory>,
    mut output_inventory_query: Query<&mut OutputInventory>,
    mut crafting_state_query: Query<(&mut CraftingState, &ActiveRecipe)>,
    recipe_manifest: Res<RecipeManifest>,
    mut rng: ResMut<GlobalRng>,
) {
    for mut energy_pool in energy_pool_query.iter_mut() {
        energy_pool.randomize(rng.get_mut())
    }

    for mut input_inventory in input_inventory_query.iter_mut() {
        input_inventory.randomize(rng.get_mut())
    }

    for mut output_inventory in output_inventory_query.iter_mut() {
        output_inventory.randomize(rng.get_mut())
    }

    for (mut crafting_state, active_recipe) in crafting_state_query.iter_mut() {
        if let Some(recipe_id) = active_recipe.recipe_id() {
            let recipe_data = recipe_manifest.get(*recipe_id);
            crafting_state.randomize(rng.get_mut(), recipe_data);
        }
    }
}
