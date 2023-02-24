//! Generating starting terrain and organisms
use crate::enum_iter::IterableEnum;
use crate::items::ItemId;
use crate::organisms::energy::{Energy, EnergyPool};
use crate::player_interaction::clipboard::StructureData;
use crate::simulation::geometry::{Facing, TilePos};
use crate::structures::{commands::StructureCommandsExt, StructureId};
use crate::terrain::{Terrain, TerrainBundle};
use crate::units::hunger::Diet;
use crate::units::UnitBundle;
use bevy::app::{App, Plugin, StartupStage};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::math::vec2;
use bevy::utils::HashMap;
use hexx::shapes::hexagon;
use hexx::Hex;
use leafwing_abilities::prelude::Pool;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::geometry::MapGeometry;

/// Controls world generation strategy
#[derive(Resource, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    map_radius: u32,
    /// Initial number of ants.
    n_ant: usize,
    /// Initial number of plants.
    n_plant: usize,
    /// Initial number of fungi.
    n_fungi: usize,
    /// Initial number of ant hives.
    n_hive: usize,
    /// Relative probability of generating tiles of each terrain type.
    terrain_weights: HashMap<Terrain, f32>,
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
    const TERRAIN_WEIGHT_PLAIN: f32 = 1.0;
    /// The choice weight for high terrain in default generation config
    const TERRAIN_WEIGHT_HIGH: f32 = 0.3;
    /// The choice weight for impassable terrain in default generation config
    const TERRAIN_WEIGHT_ROCKY: f32 = 0.2;
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<Terrain, f32> = HashMap::new();
        terrain_weights.insert(Terrain::Plain, GenerationConfig::TERRAIN_WEIGHT_PLAIN);
        terrain_weights.insert(Terrain::Muddy, GenerationConfig::TERRAIN_WEIGHT_HIGH);
        terrain_weights.insert(Terrain::Rocky, GenerationConfig::TERRAIN_WEIGHT_ROCKY);

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

/// Stage labels required to organize our startup systems.
///
/// We must use stage labels, as we need commands to be flushed between each stage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub(crate) enum GenerationStage {
    /// Randomly generates and inserts terrain entities based on the [`GenerationConfig`] resource
    ///
    /// Systems:
    /// * [`generate_terrain`]
    TerrainGeneration,
    /// Generates starting organisms, based on [`GenerationConfig`] resource, with random positions
    ///
    /// Systems:
    /// * [`generate_organisms`]
    OrganismGeneration,
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Generation plugin...");
        app.insert_resource(self.config.clone())
            .insert_resource(MapGeometry::new(self.config.map_radius))
            .add_startup_stage_before(
                StartupStage::Startup,
                GenerationStage::OrganismGeneration,
                SystemStage::parallel(),
            )
            .add_startup_stage_before(
                GenerationStage::OrganismGeneration,
                GenerationStage::TerrainGeneration,
                SystemStage::parallel(),
            )
            .add_startup_system_to_stage(GenerationStage::TerrainGeneration, generate_terrain)
            .add_startup_system_to_stage(GenerationStage::OrganismGeneration, generate_organisms);
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
    mut map_geometry: ResMut<MapGeometry>,
) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_variants = Terrain::variants().collect::<Vec<Terrain>>();
    let terrain_weights = &config.terrain_weights;

    for hex in hexagon(Hex::ZERO, map_geometry.radius) {
        let &terrain_type = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights.get(terrain_type).unwrap()
            })
            .unwrap();

        let tile_pos = TilePos { hex };
        let pos = vec2(tile_pos.x as f32, tile_pos.y as f32);

        let hex_height = MIN_HEIGHT
            + (fbm_simplex_2d_seeded(pos * FREQUENCY_SCALE, OCTAVES, LACUNARITY, GAIN, SEED)
                * AMPLITUDE_SCALE)
                .abs()
                // Height is stepped, and should always be a multiple of 1.0
                .round();

        // Spawn the terrain entity
        let terrain_entity = commands
            .spawn(TerrainBundle::new(terrain_type, tile_pos))
            .id();

        // Update the index
        map_geometry.height_index.insert(tile_pos, hex_height);
        map_geometry.terrain_index.insert(tile_pos, terrain_entity);
    }
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    tile_query: Query<&TilePos, With<Terrain>>,
) {
    info!("Generating organisms...");
    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;
    let n_hive = config.n_hive;

    let n_entities = n_ant + n_plant + n_fungi + n_hive;
    assert!(n_entities <= tile_query.iter().len());

    let mut entity_positions: Vec<TilePos> = {
        let possible_positions: Vec<TilePos> = tile_query.iter().copied().collect();

        let mut rng = &mut thread_rng();
        possible_positions
            .choose_multiple(&mut rng, n_entities)
            .cloned()
            .collect()
    };

    // Ant
    let ant_positions = entity_positions.split_off(entity_positions.len() - n_ant);
    commands.spawn_batch(
        ant_positions
            .into_iter()
            // TODO: use a UnitManifest
            .map(|ant_position| {
                UnitBundle::new(
                    "ant",
                    ant_position,
                    EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    Diet::new(ItemId::leuco_chunk(), Energy(50.)),
                )
            }),
    );

    // Plant
    let plant_positions = entity_positions.split_off(entity_positions.len() - n_plant);
    for position in plant_positions {
        let item = StructureData {
            structure_id: StructureId { id: "acacia" },
            facing: Facing::default(),
        };

        commands.spawn_structure(position, item);
    }

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    for position in fungus_positions {
        let item = StructureData {
            structure_id: StructureId { id: "leuco" },
            facing: Facing::default(),
        };

        commands.spawn_structure(position, item);
    }

    // Hives
    let hive_positions = entity_positions.split_off(entity_positions.len() - n_hive);
    for position in hive_positions {
        let item = StructureData {
            structure_id: StructureId { id: "ant_hive" },
            facing: Facing::default(),
        };

        commands.spawn_structure(position, item);
    }
}
