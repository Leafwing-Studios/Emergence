use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .add_startup_system(spawn_camera)
            .add_startup_system(spawn_tilemap);
    }
}

enum MapId {
    Main,
}

enum LayerId {
    Ground,
}

fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera");
    commands.spawn_bundle(Camera2dBundle::default());
}

fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 16.0 };
    let tilemap_size = TilemapSize { x: 16, y: 16 };
    info!("Loading texture");
    let texture_handle = asset_server.load("tiles.png");

    let tilemap_entity = commands.spawn().id();
    let mut tilemap_storage = TileStorage::empty(tilemap_size);

    info!("Populating tilemap storage");
    fill_tilemap_rect(// The texture to fill the region with
                      TileTexture(0),
                      // Position of the anchor tile defining the region
                      TilePos {x: 0, y: 0},
                      // Size of the region to fill
                      tilemap_size,
                      TilemapId(tilemap_entity),
                      &mut commands,
                      &mut tilemap_storage);

    info!("Inserting TilemapBundle");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tilemap_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..Default::default()
        });
}
