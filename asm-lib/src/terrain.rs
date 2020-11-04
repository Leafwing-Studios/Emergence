use crate::config::TILE_SIZE;
use crate::utils::Position;
use bevy::prelude::*;

struct Tile {}

pub fn build_tile(
    position: Position,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    let scale = TILE_SIZE as f32;
    let screen_x = position.x as f32 * scale;
    let screen_y = position.y as f32 * scale;

    let handle = asset_server.get_handle("tile.png");

    (
        Tile {},
        position,
        SpriteComponents {
            material: materials.add(handle.into()),
            transform: Transform::from_translation(Vec3::new(screen_x, screen_y, 0.0)),
            sprite: Sprite::new(Vec2::new(scale, scale)),
            ..Default::default()
        },
    )
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(render_terrain.system());
    }
}

fn render_terrain(_tile: &Tile, position: &Position) {
    println!("Tile: ({}, {})", position.x, position.y);
}
