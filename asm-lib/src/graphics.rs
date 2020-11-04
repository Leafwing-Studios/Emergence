use bevy::prelude::*;

use crate::config::{MAP_SIZE, TILE_SIZE};
use crate::utils::Position;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system());
    }
}

fn setup_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let _assets = asset_server.load_folder("");

    commands.spawn(Camera2dComponents::default());
}

pub fn make_sprite_components(
    position: &Position,
    handle: Handle<ColorMaterial>,
    scale: f32,
) -> impl Bundle {
    // Scaling factor for vertical compression of hexes
    const SQRT3_OVER_2: f32 = 0.866;

    // Offset odd rows
    let screen_x;
    if position.y % 2 == 0 {
        screen_x = (position.x as f32 - (0.5 * MAP_SIZE as f32)) * TILE_SIZE as f32;
    } else {
        screen_x = (position.x as f32 - (0.5 * MAP_SIZE as f32)) * TILE_SIZE as f32
            + 0.5 * TILE_SIZE as f32;
    }

    let screen_y = (position.y as f32 - (0.5 * MAP_SIZE as f32)) * TILE_SIZE as f32 * SQRT3_OVER_2;

    SpriteComponents {
        material: handle,
        transform: Transform::from_translation(Vec3::new(screen_x, screen_y, 0.0)),
        sprite: Sprite::new(Vec2::new(
            scale * TILE_SIZE as f32,
            scale * TILE_SIZE as f32,
        )),
        ..Default::default()
    }
}
