use bevy::prelude::*;

use crate::config::{TILE_BUFFER, TILE_SIZE};
use crate::position::Position;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system())
            .add_system(update_positions.system());
    }
}

fn setup_graphics(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let _assets = asset_server.load_folder("");

    commands.spawn(Camera2dComponents::default());
}

pub fn position_to_pixels(position: &Position) -> Transform {
    const SQRT_3: f32 = 1.73205080757;
    let (alpha, beta) = (position.alpha as f32, position.beta as f32);
    let scale = (0.5 + TILE_BUFFER) * TILE_SIZE as f32;

    let x = scale * (SQRT_3 * alpha + SQRT_3 / 2.0 * beta);
    let y = scale * (3.0 / 2.0 * beta);

    Transform::from_translation(Vec3::new(x, y, 0.0))
}

pub fn make_sprite_components(position: &Position, handle: Handle<ColorMaterial>) -> impl Bundle {
    SpriteComponents {
        material: handle,
        transform: position_to_pixels(position),
        sprite: Sprite::new(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
        ..Default::default()
    }
}

fn update_positions(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        *transform = position_to_pixels(position);
    }
}
