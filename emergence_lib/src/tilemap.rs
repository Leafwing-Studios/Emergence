use crate::config::{GRID_SIZE, TILE_PNG, TILE_SIZE};
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
                    .with_system(spawn_tilemap),
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_labels);
    }
}

fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera");
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam::default());
}

#[derive(Component)]
pub struct MainTilemap;

const TILEMAP_SIZE: TilemapSize = TilemapSize { x: 2, y: 2 };

fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let tile_size = TilemapTileSize { x: 42.0, y: 48.0 };
    // let grid_size = TilemapGridSize { x: 42.0, y: 48.0 };
    let tile_size = TILE_SIZE;
    let grid_size = GRID_SIZE;
    let tilemap_size = TILEMAP_SIZE;
    info!("Loading texture");
    let texture_handle = asset_server.load(TILE_PNG);

    let tilemap_entity = commands.spawn().id();
    let mut tilemap_storage = TileStorage::empty(tilemap_size);

    info!("Populating tilemap storage");
    fill_tilemap_rect(
        // The texture to fill the region with
        TileTexture(0),
        // Position of the anchor tile defining the region
        TilePos { x: 0, y: 0 },
        // Size of the region to fill
        tilemap_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tilemap_storage,
    );

    info!("Inserting TilemapBundle");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tilemap_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&tilemap_size, &tile_size, 0.0),
            map_type: TilemapType::Hexagon(HexCoordSystem::RowEven),
            ..Default::default()
        })
        .insert(MainTilemap);
}

fn spawn_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage), With<MainTilemap>>,
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
        for tile_entity in tilemap_storage.iter() {
            let tile_pos = tile_q.get(tile_entity.unwrap()).unwrap();
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
