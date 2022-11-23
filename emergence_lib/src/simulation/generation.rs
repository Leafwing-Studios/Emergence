//! Generating starting terrain and organisms
use crate::organisms::structures::{FungiBundle, PlantBundle};
use crate::organisms::units::AntBundle;
use crate::simulation::map::resources::MapResource;
use crate::simulation::map::{configure_map_geometry, populate_position_cache, MapPositions};
use crate::simulation::pathfinding::Impassable;
use crate::terrain::entity_map::TerrainEntityMap;
use crate::terrain::TerrainType;
use bevy::app::{App, Plugin, StartupStage};
use bevy::ecs::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_ecs_tilemap::tiles::TilePos;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// The number of tiles from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

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
    /// Relative probability of generating graphics of each terrain type.
    pub terrain_weights: HashMap<TerrainType, f32>,
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<TerrainType, f32> = HashMap::new();
        terrain_weights.insert(TerrainType::Plain, 1.);
        terrain_weights.insert(TerrainType::High, 0.3);
        terrain_weights.insert(TerrainType::Impassable, 0.2);

        GenerationConfig {
            map_radius: MAP_RADIUS,
            n_ant: 5,
            n_plant: 7,
            n_fungi: 4,
            terrain_weights,
        }
    }
}

/// Generate the world.
pub struct GenerationPlugin;

/// Stage labels required to organize our startup systems.
///
/// We must use stage labels, as we need commands to be flushed between each stage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub enum GenerationStage {
    /// Creates and inserts the [`MapGeometry`] resource based on the [`GenerationConfig`] resource
    ///
    /// Systems:
    /// * [`configure_map_geometry`]
    Configuration,
    /// Creates and inserts the [`MapPositionCache`] resource.
    ///
    /// Systems:
    /// * [`populate_position_cache`]
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

/// Label for the "grand stage" in which all the [`GenerationStage`]s are executed
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub struct GenerationScheduleLabel;

impl GenerationScheduleLabel {
    /// Creates a schedule out of stages enumerated in [`GenerationStage`]
    pub fn schedule() -> Schedule {
        Schedule::default()
            .with_stage(GenerationStage::Configuration, SystemStage::parallel())
            .with_stage_after(
                GenerationStage::Configuration,
                GenerationStage::PositionCaching,
                SystemStage::parallel(),
            )
            .with_stage_after(
                GenerationStage::PositionCaching,
                GenerationStage::TerrainGeneration,
                SystemStage::parallel(),
            )
            .with_stage_after(
                GenerationStage::TerrainGeneration,
                GenerationStage::OrganismGeneration,
                SystemStage::parallel(),
            )
    }
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationConfig>()
            .add_stage_before(
                StartupStage::Startup,
                GenerationScheduleLabel,
                GenerationScheduleLabel::schedule(),
            )
            // .init_resource::<PositionCache>()
            // This inserts the `MapGeometry` resource, and so needs to run in an earlier stage
            .add_startup_system_to_stage(GenerationStage::Configuration, configure_map_geometry)
            .add_startup_system_to_stage(GenerationStage::PositionCaching, populate_position_cache)
            .add_startup_system_to_stage(GenerationStage::TerrainGeneration, generate_terrain)
            .add_startup_system_to_stage(GenerationStage::OrganismGeneration, generate_organisms);
    }
}

/// Creates the world according to [`GenerationConfig`].
fn generate_terrain(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    map_positions: Res<MapPositions>,
) {
    let mut rng = thread_rng();

    let mut entity_data = Vec::with_capacity(map_positions.n_positions());
    for position in map_positions.iter_positions() {
        let terrain: TerrainType =
            TerrainType::choose_random(&mut rng, &config.terrain_weights).unwrap();
        entity_data.push((*position, terrain.instantiate(&mut commands, position)));
    }

    let terrain_entities = TerrainEntityMap {
        inner: MapResource::new(&map_positions, entity_data.into_iter()),
    };
    commands.insert_resource(terrain_entities)
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    passable_tiles: Query<&TilePos, Without<Impassable>>,
) {
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
    commands.spawn_batch(plant_positions.into_iter().map(PlantBundle::new));

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    commands.spawn_batch(fungus_positions.into_iter().map(FungiBundle::new));
}
