//! Tools and strategies for procedural world generation.
use crate::graphics::{GRID_SIZE, MAP_COORD_SYSTEM, MAP_TYPE};
use crate::organisms::units::AntBundle;
use crate::terrain::terrain_types::{ImpassableTerrain, TerrainType};
use crate::terrain::MAP_RADIUS;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::*;

use rand::prelude::*;

use crate::graphics::organisms::OrganismTilemap;
use crate::graphics::terrain::TerrainTilemap;
use crate::organisms::structures::{FungiBundle, PlantBundle};
use config::*;

use super::MapGeometry;

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
    info!("Generating terrain tilemap...");
    let texture = TilemapTexture::Vector(todo!());

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(map_geometry.size());

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

/// Generate debug labels for tile positions
fn generate_debug_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TileStorage),
        With<TerrainTilemap>,
    >,
    tile_q: Query<&mut TilePos>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: 15.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (tilemap_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        for tile_entity in tilemap_storage.iter().filter_map(|e| e.as_ref()) {
            if let Ok(tile_pos) = tile_q.get(*tile_entity) {
                let tile_pos_transform = Transform::from_translation(
                    tile_pos.center_in_world(grid_size, map_type).extend(1.0),
                );
                let transform = *tilemap_transform * tile_pos_transform;
                commands.spawn_bundle(Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..default()
                });
            }
        }
    }
}
