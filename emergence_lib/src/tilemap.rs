use crate::config::{GRID_SIZE, MAP_COORD_SYSTEM, MAP_SIZE, TERRAIN_TILE_IMAP, TERRAIN_TILE_SIZE};
use crate::terrain::generate_simple_random_terrain;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapType;
use bevy_ecs_tilemap::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .add_plugin(PanCamPlugin)
            .add_startup_system_set_to_stage(
                StartupStage::Startup,
                SystemSet::new()
                    .with_system(spawn_camera)
                    .with_system(spawn_terrain_tilemap),
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_labels);
    }
}

fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera...");
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam::default());
}

#[derive(Component)]
pub struct TerrainTilemap;

#[derive(Component)]
pub struct OrganismTilemap;

fn spawn_terrain_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning terrain tilemap...");
    let texture = TilemapTexture::Vector(
        (&TERRAIN_TILE_IMAP)
            .values()
            .map(|&p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(MAP_SIZE);

    info!("Generating simple random terrain...");
    generate_simple_random_terrain(&mut commands, TilemapId(tilemap_entity), &mut tile_storage);

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: GRID_SIZE,
            size: MAP_SIZE,
            storage: tile_storage,
            texture,
            tile_size: TERRAIN_TILE_SIZE,
            transform: get_tilemap_center_transform(&MAP_SIZE, &GRID_SIZE, 0.0),
            map_type: TilemapType::Hexagon(MAP_COORD_SYSTEM),
            ..Default::default()
        })
        .insert(TerrainTilemap);
}

// fn spawn_organism_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
//     info!("Spawning terrain tilemap...");
//     let texture = TilemapTexture::Vector(
//         (&ORGANISM_TILE_IMAP)
//             .values()
//             .map(|&p| asset_server.load(p))
//             .collect(),
//     );
//
//     let tilemap_entity = commands.spawn().id();
//     let mut tile_storage = TileStorage::empty(MAP_SIZE);
//
//     info!("Generating starting organisms...");
//     generate_starting_organisms(&mut commands, TilemapId(tilemap_entity), &mut tile_storage);
//
//     info!("Inserting TilemapBundle...");
//     commands
//         .entity(tilemap_entity)
//         .insert_bundle(TilemapBundle {
//             grid_size: GRID_SIZE,
//             size: MAP_SIZE,
//             storage: tile_storage,
//             texture,
//             tile_size: ORGANISM_TILE_SIZE,
//             transform: get_tilemap_center_transform(&MAP_SIZE, &GRID_SIZE, 0.0),
//             map_type: TilemapType::Hexagon(MAP_COORD_SYSTEM),
//             ..Default::default()
//         })
//         .insert(OrganismTilemap);
// }

fn spawn_labels(
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
