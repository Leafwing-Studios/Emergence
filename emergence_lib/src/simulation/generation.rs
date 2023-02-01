//! Generating starting terrain and organisms
use crate::enum_iter::IterableEnum;
use crate::organisms::sessile::fungi::LeucoBundle;
use crate::organisms::sessile::plants::AcaciaBundle;
use crate::organisms::units::AntBundle;
use crate::terrain::TerrainType;
use bevy::app::{App, Plugin, StartupStage};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::utils::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::map::TilePos;

/// Controls world generation strategy
#[derive(Resource, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub map_radius: u32,
    /// Initial number of ants.
    pub n_ant: usize,
    /// Initial number of plants.
    pub n_plant: usize,
    /// Initial number of fungi.
    pub n_fungi: usize,
    /// Relative probability of generating tiles of each terrain type.
    pub terrain_weights: HashMap<TerrainType, f32>,
}

impl GenerationConfig {
    /// The number of tiles from the center of the map to the edge
    pub const MAP_RADIUS: u32 = 20;

    /// The number of ants in the default generation config
    pub const N_ANT: usize = 5;
    /// The number of plants in the default generation config
    pub const N_PLANT: usize = 7;
    /// The number of fungi in the default generation config
    pub const N_FUNGI: usize = 4;

    /// The choice weight for plain terrain in default generation config
    pub const TERRAIN_WEIGHT_PLAIN: f32 = 1.0;
    /// The choice weight for high terrain in default generation config
    pub const TERRAIN_WEIGHT_HIGH: f32 = 0.3;
    /// The choice weight for impassable terrain in default generation config
    pub const TERRAIN_WEIGHT_ROCKY: f32 = 0.2;
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<TerrainType, f32> = HashMap::new();
        terrain_weights.insert(TerrainType::Plain, GenerationConfig::TERRAIN_WEIGHT_PLAIN);
        terrain_weights.insert(TerrainType::High, GenerationConfig::TERRAIN_WEIGHT_HIGH);
        terrain_weights.insert(TerrainType::Rocky, GenerationConfig::TERRAIN_WEIGHT_ROCKY);

        GenerationConfig {
            map_radius: GenerationConfig::MAP_RADIUS,
            n_ant: GenerationConfig::N_ANT,
            n_plant: GenerationConfig::N_PLANT,
            n_fungi: GenerationConfig::N_FUNGI,
            terrain_weights,
        }
    }
}

/// Generate the world.
pub struct GenerationPlugin {
    /// Configuration settings for world generation
    pub config: GenerationConfig,
}

/// Stage labels required to organize our startup systems.
///
/// We must use stage labels, as we need commands to be flushed between each stage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub enum GenerationStage {
    /// Creates and inserts the [`MapGeometry`](crate::simulation::map::MapGeometry) resource based on the [`GenerationConfig`] resource
    ///
    /// Systems:
    /// * [`configure_map_geometry`]
    Configuration,
    /// Creates and inserts the [`MapPositions`] resource.
    ///
    /// Systems:
    /// * [`create_map_positions`]
    PositionCaching,
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
            .add_startup_stage_before(
                GenerationStage::TerrainGeneration,
                GenerationStage::PositionCaching,
                SystemStage::parallel(),
            )
            .add_startup_stage_before(
                GenerationStage::PositionCaching,
                GenerationStage::Configuration,
                SystemStage::parallel(),
            )
            .add_startup_system_to_stage(GenerationStage::TerrainGeneration, generate_terrain)
            .add_startup_system_to_stage(GenerationStage::OrganismGeneration, generate_organisms);
    }
}

/// Creates the world according to [`GenerationConfig`].
pub fn generate_terrain(mut commands: Commands, config: Res<GenerationConfig>) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_variants = TerrainType::variants().collect::<Vec<TerrainType>>();
    let terrain_weights = &config.terrain_weights;

    let entity_data = map_positions.iter_positions().map(|position| {
        let terrain: TerrainType = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights
                    .get(terrain_type)
                    .copied()
                    .unwrap_or_default()
            })
            .copied()
            .unwrap();
        (*position, terrain.instantiate(&mut commands, position))
    });
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
pub fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    passable_tiles: Query<&TilePos, Without<Impassable>>,
) {
    info!("Generating organisms...");
    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;

    let n_entities = n_ant + n_plant + n_fungi;

    let mut entity_positions: Vec<TilePos> = {
        let possible_positions: Vec<TilePos> = passable_tiles.iter().copied().collect();

        let mut rng = &mut thread_rng();
        possible_positions
            .choose_multiple(&mut rng, n_entities)
            .cloned()
            .collect()
    };

    // Ant
    let ant_positions = entity_positions.split_off(entity_positions.len() - n_ant);
    commands.spawn_batch(ant_positions.into_iter().map(AntBundle::new));

    // Plant
    let plant_positions = entity_positions.split_off(entity_positions.len() - n_plant);
    commands.spawn_batch(plant_positions.into_iter().map(AcaciaBundle::new));

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    commands.spawn_batch(fungus_positions.into_iter().map(LeucoBundle::new));
}
