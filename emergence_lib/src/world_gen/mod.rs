//! Generating starting terrain and organisms
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetState;
use crate::crafting::components::{ActiveRecipe, CraftingState, InputInventory, OutputInventory};
use crate::crafting::recipe::RecipeManifest;
use crate::organisms::energy::EnergyPool;
use crate::player_interaction::clipboard::ClipboardData;
use crate::simulation::geometry::{Facing, Height, MapGeometry, TilePos};
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::{Structure, StructureManifest};
use crate::terrain::commands::TerrainCommandsExt;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_assets::UnitHandles;
use crate::units::unit_manifest::{Unit, UnitManifest};
use crate::units::UnitBundle;
use crate::water::WaterTable;
use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::IntoSystemAppConfigs;
use bevy::utils::HashMap;
use hexx::shapes::hexagon;
use hexx::Hex;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

/// Generate the world.
pub(super) struct GenerationPlugin {
    /// Configuration settings for world generation
    pub(super) config: GenerationConfig,
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Generation plugin...");
        app.insert_resource(self.config.clone()).add_systems(
            (
                generate_terrain,
                apply_system_buffers,
                generate_landmarks,
                initialize_water_table,
                apply_system_buffers,
                generate_organisms,
                apply_system_buffers,
                randomize_starting_organisms,
            )
                .chain()
                .in_schedule(OnEnter(AssetState::FullyLoaded)),
        );
    }
}

/// Controls world generation strategy
#[derive(Resource, Debug, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub(super) map_radius: u32,
    /// Chance that each tile contains a landmark of the given type.
    landmark_chances: HashMap<Id<Structure>, f32>,
    /// Chance that each tile contains a unit of the given type.
    unit_chances: HashMap<Id<Unit>, f32>,
    /// Chance that each tile contains a structure of the given type.
    structure_chances: HashMap<Id<Structure>, f32>,
    /// Relative probability of generating tiles of each terrain type.
    terrain_weights: HashMap<Id<Terrain>, f32>,
    /// Controls the noise added to produce the larger land forms.
    low_frequency_noise: SimplexSettings,
    /// Controls the noise added to the terrain heights.
    high_frequency_noise: SimplexSettings,
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("loam".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("muddy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let mut landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        landmark_chances.insert(Id::from_name("spring".to_string()), 0.2);

        let mut unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();
        unit_chances.insert(Id::from_name("ant".to_string()), 1e-2);

        let mut structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        structure_chances.insert(Id::from_name("ant_hive".to_string()), 1e-3);
        structure_chances.insert(Id::from_name("acacia".to_string()), 2e-2);
        structure_chances.insert(Id::from_name("leuco".to_string()), 1e-2);

        GenerationConfig {
            map_radius: 40,
            unit_chances,
            landmark_chances,
            structure_chances,
            terrain_weights,
            low_frequency_noise: SimplexSettings {
                frequency: 0.02,
                amplitude: 5.0,
                octaves: 4,
                lacunarity: 2.3,
                gain: 0.5,
                seed: 2378.0,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 1.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
                seed: 100.0,
            },
        }
    }
}

/// Creates the world according to [`GenerationConfig`].
pub(crate) fn generate_terrain(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_weights = &generation_config.terrain_weights;
    let terrain_variants: Vec<Id<Terrain>> = terrain_weights.keys().copied().collect();

    for hex in hexagon(Hex::ZERO, map_geometry.radius) {
        // FIXME: can we not just sample from our terrain_weights directly?
        let &terrain_id = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights.get(terrain_type).unwrap()
            })
            .unwrap();

        let tile_pos = TilePos { hex };
        // Heights are generated in f32 world coordinates to start
        let hex_height = simplex_noise(tile_pos, &generation_config.low_frequency_noise)
            + simplex_noise(tile_pos, &generation_config.high_frequency_noise);

        // And then discretized to the nearest integer height before being used
        let height = Height::from_world_pos(hex_height);

        commands.spawn_terrain(tile_pos, height, terrain_id);
    }
}

/// A settings struct for [`simplex_noise`].
#[derive(Debug, Clone)]
struct SimplexSettings {
    /// Controls the size of the features in the noise function.
    ///
    /// Higher values mean smaller features.
    frequency: f32,
    /// Controls the vertical scale of the noise function.
    ///
    /// Higher values mean deeper valleys and higher mountains.
    amplitude: f32,
    /// How many times will the fbm be sampled?
    octaves: usize,
    /// Smoothing factor
    lacunarity: f32,
    /// Scale the output of the fbm function
    gain: f32,
    /// Arbitary seed that determines the noise function output
    seed: f32,
}

/// Computes the value of the noise function at a given position.
///
/// This can then be used to determine the height of a tile.
fn simplex_noise(tile_pos: TilePos, settings: &SimplexSettings) -> f32 {
    let SimplexSettings {
        frequency,
        amplitude,
        octaves,
        lacunarity,
        gain,
        seed,
    } = *settings;

    let pos = Vec2::new(tile_pos.hex.x as f32, tile_pos.hex.y as f32);

    Height::MIN.into_world_pos()
        + (fbm_simplex_2d_seeded(pos * frequency, octaves, lacunarity, gain, seed) * amplitude)
            .abs()
}

/// Places landmarks according to [`GenerationConfig`].
fn generate_landmarks(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    structure_manifest: Res<StructureManifest>,
    mut height_query: Query<&mut Height>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    info!("Generating landmarks...");
    let rng = &mut thread_rng();

    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        for (&structure_id, &chance) in &generation_config.landmark_chances {
            if rng.gen::<f32>() < chance {
                let mut clipboard_data =
                    ClipboardData::generate_from_id(structure_id, &structure_manifest);
                let facing = Facing::random(rng);
                clipboard_data.facing = facing;
                let footprint = &structure_manifest.get(structure_id).footprint;

                // Only try to spawn a structure if the location is valid and there is space
                if map_geometry.is_footprint_valid(tile_pos, footprint, &facing)
                    && map_geometry.is_space_available(tile_pos, footprint, &facing)
                {
                    // Flatten the terrain under the structure before spawning it
                    map_geometry.flatten_height(&mut height_query, tile_pos, footprint, &facing);
                    commands.spawn_structure(
                        tile_pos,
                        ClipboardData::generate_from_id(structure_id, &structure_manifest),
                    );
                }
            }
        }
    }
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
fn generate_organisms(
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
                if map_geometry.is_footprint_valid(tile_pos, footprint, &facing)
                    && map_geometry.is_space_available(tile_pos, footprint, &facing)
                {
                    // Flatten the terrain under the structure before spawning it
                    map_geometry.flatten_height(&mut height_query, tile_pos, footprint, &facing);
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
fn randomize_starting_organisms(
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

/// Sets the starting water table
fn initialize_water_table(mut water_table: ResMut<WaterTable>, map_geometry: Res<MapGeometry>) {
    /// The minimum starting water level for low lying areas
    const LOW_WATER_LINE: Height = Height(2.0);

    /// Scales the distance of the water table from the surface of the soil
    const DISTANCE_FROM_SURFACE: Height = Height(1.5);

    for tile_pos in map_geometry.valid_tile_positions() {
        let height = map_geometry.get_height(tile_pos).unwrap();
        let water_table_level = if height < LOW_WATER_LINE {
            LOW_WATER_LINE
        } else {
            LOW_WATER_LINE + (height - DISTANCE_FROM_SURFACE) / 2.0
        };

        water_table.set(tile_pos, water_table_level);
    }
}
