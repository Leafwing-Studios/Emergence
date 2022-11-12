use crate::graphics::MAP_COORD_SYSTEM;
use crate::terrain::{terrain_type::TerrainType, MapGeometry};
use bevy::app::{App, Plugin, StartupStage};
use bevy::ecs::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::{generate_hexagon, TilemapGridSize};
use bevy_ecs_tilemap::tiles::TilePos;
use std::collections::HashMap;

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

/// Controls world generation strategy
#[derive(Clone)]
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

/// Generate the world.
pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationConfig>()
            // .init_resource::<PositionCache>()
            // This inserts the `MapGeometry` resource, and so needs to run in an earlier stage
            .add_startup_system_to_stage(StartupStage::PreStartup, configure_map_geometry)
            .add_startup_system_to_stage(StartupStage::Startup, generate_terrain)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_starting_organisms)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_debug_labels);
    }
}

// pub struct PositionCache {
//     grid_positions: HashMap<TilePos, HexNeighbors<TilePos>>,
// }

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<TerrainType, f32> = HashMap::new();
        terrain_weights.insert(TerrainType::Plain, 1.);
        terrain_weights.insert(TerrainType::High, 0.3);
        terrain_weights.insert(TerrainType::Impassable, 0.2);

        GenerationConfig {
            map_radius: MAP_RADIUS,
            n_ant: N_ANT,
            n_plant: N_PLANT,
            n_fungi: N_FUNGI,
            terrain_weights,
        }
    }
}

impl From<&GenerationConfig> for MapGeometry {
    fn from(config: &GenerationConfig) -> MapGeometry {
        let diameter = 2 * config.map_radius + 1;
        let center = config.map_radius + 1;
        MapGeometry {
            radius: config.map_radius,
            center: TilePos {
                x: center,
                y: center,
            },
            size: TilemapSize {
                x: diameter,
                y: diameter,
            },
        }
    }
}

/// Initialize [`MapGeometry`] according to [`GenerationConfig`].
fn configure_map_geometry(mut commands: Commands, config: Res<GenerationConfig>) {
    let map_geometry: MapGeometry = (&*config).into();

    commands.insert_resource(map_geometry);
}

/// Creates the world according to the provided [`GenerationConfig`].
fn generate_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GenerationConfig>,
    map_geometry: Res<MapGeometry>,
) {
    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&map_geometry.center(), MAP_COORD_SYSTEM),
        config.map_radius,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(MAP_COORD_SYSTEM));

    let mut rng = thread_rng();
    for position in tile_positions {
        let terrain: TerrainType =
            TerrainType::choose_random(&mut rng, &config.terrain_weights).unwrap();
        let entity = terrain.create_entity(&mut commands, TilemapId(tilemap_entity), position);
        tile_storage.set(&position, entity);
    }

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: GRID_SIZE,
            map_type: MAP_TYPE,
            size: map_geometry.size(),
            storage: tile_storage,
            texture,
            tile_size: TerrainTilemap::TILE_SIZE,
            transform: get_tilemap_center_transform(
                &map_geometry.size(),
                &GRID_SIZE,
                &MAP_TYPE,
                TerrainTilemap::MAP_Z,
            ),
            ..Default::default()
        })
        .insert(TerrainTilemap);
}

/// Randomize and place starting organisms
fn generate_starting_organisms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GenerationConfig>,
    terrain_tile_storage_query: Query<&TileStorage, With<TerrainTilemap>>,
    impassable_query: Query<&ImpassableTerrain>,
    map_geometry: Res<MapGeometry>,
) {
    let texture = TilemapTexture::Vector(todo!());

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(map_geometry.size());
    let tilemap_id = TilemapId(tilemap_entity);

    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;

    let n_entities = n_ant + n_plant + n_fungi;
    let terrain_tile_storage = terrain_tile_storage_query.single();

    let mut entity_positions: Vec<TilePos> = {
        let possible_positions = generate_hexagon(
            AxialPos::from_tile_pos_given_coord_system(&map_geometry.center(), MAP_COORD_SYSTEM),
            config.map_radius,
        )
        .into_iter()
        .filter_map(|axial_pos| {
            let tile_pos = axial_pos.as_tile_pos_given_coord_system(MAP_COORD_SYSTEM);
            terrain_tile_storage.get(&tile_pos).and_then(|entity| {
                if impassable_query.get(entity).is_err() {
                    Some(tile_pos)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<TilePos>>();

        let mut rng = &mut thread_rng();
        possible_positions
            .choose_multiple(&mut rng, n_entities)
            .cloned()
            .collect()
    };

    // PERF: Swap this to spawn_batch

    // Ant
    let ant_positions = entity_positions.split_off(entity_positions.len() - n_ant);
    for position in ant_positions {
        let entity = commands
            .spawn_bundle(AntBundle::new(tilemap_id, position))
            .id();
        tile_storage.set(&position, entity);
    }

    // Plant
    let plant_positions = entity_positions.split_off(entity_positions.len() - n_plant);
    for position in plant_positions {
        let entity = commands
            .spawn_bundle(PlantBundle::new(tilemap_id, position))
            .id();
        tile_storage.set(&position, entity);
    }

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    for position in fungus_positions {
        let entity = commands
            .spawn_bundle(FungiBundle::new(tilemap_id, position))
            .id();
        tile_storage.set(&position, entity);
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: GRID_SIZE,
            map_type: MAP_TYPE,
            size: map_geometry.size(),
            storage: tile_storage,
            texture,
            tile_size: OrganismTilemap::TILE_SIZE,
            transform: get_tilemap_center_transform(
                &map_geometry.size(),
                &GRID_SIZE,
                &MAP_TYPE,
                OrganismTilemap::MAP_Z,
            ),
            ..Default::default()
        })
        .insert(OrganismTilemap);
}
