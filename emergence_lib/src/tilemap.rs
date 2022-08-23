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

const TILE_SIZE_ROW: TilemapTileSize = TilemapTileSize { x: 42.0, y: 48.0 };
const GRID_SIZE_ROW: TilemapGridSize = TilemapGridSize { x: 42.0, y: 48.0 };
const TILE_SIZE_COL: TilemapTileSize = TilemapTileSize { x: 48.0, y: 42.0 };
const GRID_SIZE_COL: TilemapGridSize = TilemapGridSize { x: 48.0, y: 42.0 };

const TILEMAP_SIZE: TilemapSize = TilemapSize { x: 2, y: 2 };
const ROW_TILE_PNG: &'static str = "tile2.png";
const COL_TILE_PNG: &'static str = "tile3.png";

fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let tile_size = TilemapTileSize { x: 42.0, y: 48.0 };
    // let grid_size = TilemapGridSize { x: 42.0, y: 48.0 };
    let tile_size = TILE_SIZE_ROW;
    let grid_size = GRID_SIZE_ROW;
    let tilemap_size = TILEMAP_SIZE;
    info!("Loading texture");
    let texture_handle = asset_server.load(ROW_TILE_PNG);

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
            transform: get_centered_transform_2d(&tilemap_size, &tile_size, 0.0),
            map_type: TilemapType::Hexagon(HexCoordSystem::RowEven),
            ..Default::default()
        })
        .insert(MainTilemap);
}

fn spawn_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<
        (
            &Transform,
            &TilemapType,
            &TilemapGridSize,
            &TilemapTileSize,
            &TileStorage,
        ),
        With<MainTilemap>,
    >,
    tile_q: Query<&mut TilePos>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: 10.0,
        color: Color::RED,
    };
    let text_alignment = TextAlignment::CENTER;
    for (tilemap_transform, map_type, grid_size, tile_size, tilemap_storage) in tilemap_q.iter() {
        let tile_size: Vec2 = (*tile_size).into();
        for tile_entity in tilemap_storage.iter() {
            let tile_pos = tile_q.get(tile_entity.unwrap()).unwrap();
            let tile_pos_in_px = get_tile_pos_in_world_space(tile_pos, grid_size, map_type);
            let mut transform = tilemap_transform.clone();
            transform.translation += (tile_pos_in_px + 0.5 * tile_size).extend(2.0);
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
