//! Generating starting terrain and organisms
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetState;
use crate::player_interaction::clipboard::ClipboardData;
use crate::simulation::geometry::{Facing, Height, TilePos};
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::StructureManifest;
use crate::terrain::commands::TerrainCommandsExt;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_assets::UnitHandles;
use crate::units::unit_manifest::UnitManifest;
use crate::units::UnitBundle;
use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::math::vec2;
use bevy::prelude::IntoSystemAppConfigs;
use bevy::utils::HashMap;
use hexx::shapes::hexagon;
use hexx::Hex;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::geometry::MapGeometry;

/// Controls world generation strategy
#[derive(Resource, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub(super) map_radius: u32,
    /// Initial number of ants.
    n_ant: usize,
    /// Initial number of plants.
    n_plant: usize,
    /// Initial number of fungi.
    n_fungi: usize,
    /// Initial number of ant hives.
    n_hive: usize,
    /// Relative probability of generating tiles of each terrain type.
    terrain_weights: HashMap<Id<Terrain>, f32>,
}

impl GenerationConfig {
    /// The number of tiles from the center of the map to the edge
    const MAP_RADIUS: u32 = 20;

    /// The number of ants in the default generation config
    const N_ANT: usize = 5;
    /// The number of plants in the default generation config
    const N_PLANT: usize = 12;
    /// The number of fungi in the default generation config
    const N_FUNGI: usize = 2;
    /// The number of ant hives in the default generation config
    const N_HIVE: usize = 1;

    /// The choice weight for plain terrain in default generation config
    const TERRAIN_WEIGHT_LOAM: f32 = 1.0;
    /// The choice weight for high terrain in default generation config
    const TERRAIN_WEIGHT_MUDDY: f32 = 0.3;
    /// The choice weight for impassable terrain in default generation config
    const TERRAIN_WEIGHT_ROCKY: f32 = 0.2;
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(
            Id::from_name("loam".to_string()),
            GenerationConfig::TERRAIN_WEIGHT_LOAM,
        );
        terrain_weights.insert(
            Id::from_name("muddy".to_string()),
            GenerationConfig::TERRAIN_WEIGHT_MUDDY,
        );
        terrain_weights.insert(
            Id::from_name("rocky".to_string()),
            GenerationConfig::TERRAIN_WEIGHT_ROCKY,
        );

        GenerationConfig {
            map_radius: GenerationConfig::MAP_RADIUS,
            n_ant: GenerationConfig::N_ANT,
            n_plant: GenerationConfig::N_PLANT,
            n_fungi: GenerationConfig::N_FUNGI,
            n_hive: GenerationConfig::N_HIVE,
            terrain_weights,
        }
    }
}

/// Generate the world.
pub(super) struct GenerationPlugin {
    /// Configuration settings for world generation
    pub(super) config: GenerationConfig,
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Generation plugin...");
        app.insert_resource(self.config.clone()).add_systems(
            (generate_terrain, apply_system_buffers, generate_organisms)
                .chain()
                .in_schedule(OnEnter(AssetState::FullyLoaded)),
        );
    }
}

/// The minimum height of any tile.
///
/// This should always be a multiple of 1.0;
const MIN_HEIGHT: f32 = 1.0;
/// Scale the pos to make it work better with the noise function
const FREQUENCY_SCALE: f32 = 0.07;
/// Scale the output of the noise function so you can more easily use the number for a height
const AMPLITUDE_SCALE: f32 = 2.0;
/// How many times will the fbm be sampled?
const OCTAVES: usize = 4;
/// Smoothing factor
const LACUNARITY: f32 = 2.3;
/// Scale the output of the fbm function
const GAIN: f32 = 0.5;
/// Seed that determines the noise function output
const SEED: f32 = 2378.0;

/// Creates the world according to [`GenerationConfig`].
pub(crate) fn generate_terrain(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_weights = &config.terrain_weights;
    let terrain_variants: Vec<Id<Terrain>> = terrain_weights.keys().copied().collect();

    for hex in hexagon(Hex::ZERO, map_geometry.radius) {
        // FIXME: can we not just sample from our terrain_weights directly?
        let &terrain_id = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights.get(terrain_type).unwrap()
            })
            .unwrap();

        let tile_pos = TilePos { hex };
        let pos = vec2(tile_pos.x as f32, tile_pos.y as f32);

        let hex_height = MIN_HEIGHT
            + (fbm_simplex_2d_seeded(pos * FREQUENCY_SCALE, OCTAVES, LACUNARITY, GAIN, SEED)
                * AMPLITUDE_SCALE)
                .abs();

        let height = Height::from_world_pos(hex_height);

        commands.spawn_terrain(tile_pos, height, terrain_id);
    }
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    tile_query: Query<&TilePos, With<Id<Terrain>>>,
    unit_handles: Res<UnitHandles>,
    unit_manifest: Res<UnitManifest>,
    structure_manifest: Res<StructureManifest>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating organisms...");
    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;
    let n_hive = config.n_hive;

    let n_entities = n_ant + n_plant + n_fungi + n_hive;
    assert!(n_entities <= tile_query.iter().len());

    let mut rng = &mut thread_rng();
    let mut entity_positions: Vec<TilePos> = {
        let possible_positions: Vec<TilePos> = tile_query.iter().copied().collect();

        possible_positions
            .choose_multiple(&mut rng, n_entities)
            .cloned()
            .collect()
    };

    // Ant
    let ant_positions = entity_positions.split_off(entity_positions.len() - n_ant);
    for ant_position in ant_positions {
        commands.spawn(UnitBundle::new(
            Id::from_name("ant".to_string()),
            ant_position,
            unit_manifest.get(Id::from_name("ant".to_string())).clone(),
            &unit_handles,
            &map_geometry,
        ));
    }

    // Plant
    let plant_positions = entity_positions.split_off(entity_positions.len() - n_plant);
    for position in plant_positions {
        let structure_id = Id::from_name("acacia".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    for position in fungus_positions {
        let structure_id = Id::from_name("leuco".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }

    // Hives
    let hive_positions = entity_positions.split_off(entity_positions.len() - n_hive);
    for position in hive_positions {
        let structure_id = Id::from_name("ant_hive".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }
}
