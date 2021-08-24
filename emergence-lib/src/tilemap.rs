use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .add_startup_system(spawn_camera.system())
            .add_startup_system(spawn_tilemap.system());
    }
}

enum MapId {
    Main,
}

impl Into<u16> for MapId {
    fn into(self) -> u16 {
        match self {
            MapId::Main => 0u16,
        }
    }
}

enum LayerId {
    Ground,
}

impl Into<u16> for LayerId {
    fn into(self) -> u16 {
        match self {
            LayerId::Ground => 0u16,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Map creation
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Layer creation
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            // Map size in chunks
            UVec2::new(2, 2),
            // Chunk size in tiles
            UVec2::new(8, 8),
            // Tile size in pixels
            Vec2::new(16.0, 16.0),
            // Texture size in pixels (of entire spritesheet)
            Vec2::new(96.0, 256.0),
        ),
        MapId::Main,
        LayerId::Ground,
        None,
    );

    layer_builder.set_all(TileBundle::default());
    // Spawns in Tile entities for the layer
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, material_handle);

    // Store the layer inside the map
    map.add_layer(&mut commands, LayerId::Ground, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}
