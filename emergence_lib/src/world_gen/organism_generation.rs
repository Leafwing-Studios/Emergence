//! Initializes organisms in the world.

use crate::crafting::inventories::{CraftingState, InputInventory, OutputInventory};
use crate::crafting::recipe::{ActiveRecipe, RecipeManifest};
use crate::organisms::energy::EnergyPool;
use crate::player_interaction::clipboard::ClipboardData;
use crate::simulation::geometry::{Facing, Height, MapGeometry};
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::StructureManifest;
use crate::units::unit_assets::UnitHandles;
use crate::units::unit_manifest::UnitManifest;
use crate::units::UnitBundle;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use super::GenerationConfig;

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
pub(super) fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    unit_handles: Res<UnitHandles>,
    unit_manifest: Res<UnitManifest>,
    structure_manifest: Res<StructureManifest>,
    mut height_query: Query<&mut Height>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    info!("Generating organisms...");
    let rng = &mut thread_rng();

    // Collect out so we can mutate the height map to flatten the terrain while in the loop
    for tile_pos in map_geometry.valid_tile_positions().collect::<Vec<_>>() {
        for (&structure_id, &chance) in &config.structure_chances {
            if rng.gen::<f32>() < chance {
                let mut clipboard_data =
                    ClipboardData::generate_from_id(structure_id, &structure_manifest);
                let facing = Facing::random(rng);
                clipboard_data.facing = facing;
                let footprint = &structure_manifest.get(structure_id).footprint;

                // Only try to spawn a structure if the location is valid and there is space
                if map_geometry.is_footprint_valid(tile_pos, footprint, facing)
                    && map_geometry.is_space_available(tile_pos, footprint, facing)
                {
                    // Flatten the terrain under the structure before spawning it
                    map_geometry.flatten_height(&mut height_query, tile_pos, footprint, facing);
                    commands.spawn_structure(
                        tile_pos,
                        ClipboardData::generate_from_id(structure_id, &structure_manifest),
                    );
                }
            }
        }

        for (&unit_id, &chance) in &config.unit_chances {
            if rng.gen::<f32>() < chance {
                commands.spawn(UnitBundle::randomized(
                    unit_id,
                    tile_pos,
                    unit_manifest.get(unit_id).clone(),
                    &unit_handles,
                    &map_geometry,
                    rng,
                ));
            }
        }
    }
}

/// Sets all the starting organisms to a random state to avoid strange synchronization issues.
pub(super) fn randomize_starting_organisms(
    mut energy_pool_query: Query<&mut EnergyPool>,
    mut input_inventory_query: Query<&mut InputInventory>,
    mut output_inventory_query: Query<&mut OutputInventory>,
    mut crafting_state_query: Query<(&mut CraftingState, &ActiveRecipe)>,
    recipe_manifest: Res<RecipeManifest>,
) {
    let rng = &mut thread_rng();

    for mut energy_pool in energy_pool_query.iter_mut() {
        energy_pool.randomize(rng)
    }

    for mut input_inventory in input_inventory_query.iter_mut() {
        input_inventory.randomize(rng)
    }

    for mut output_inventory in output_inventory_query.iter_mut() {
        output_inventory.randomize(rng)
    }

    for (mut crafting_state, active_recipe) in crafting_state_query.iter_mut() {
        if let Some(recipe_id) = active_recipe.recipe_id() {
            let recipe_data = recipe_manifest.get(*recipe_id);
            crafting_state.randomize(rng, recipe_data);
        }
    }
}
