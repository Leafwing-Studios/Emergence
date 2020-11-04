use bevy::prelude::*;

use crate::config::{MAP_SIZE, TILE_BUFFER, TILE_SIZE};
use crate::utils::Position;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system())
            .add_system(update_positions.system());
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

pub fn position_to_pixels(position: &Position) -> Transform {
    const SQRT_3: f32 = 1.73205080757;
    let (alpha, beta) = (position.alpha as f32, position.beta as f32);
    let scale = (0.5 + TILE_BUFFER) * TILE_SIZE as f32;

    let x = scale * (SQRT_3 * alpha + SQRT_3 / 2.0 * beta);
    let y = scale * (3.0 / 2.0 * beta);

    Transform::from_translation(Vec3::new(x, y, 0.0))
}

pub fn make_sprite_components(
    position: &Position,
    handle: Handle<ColorMaterial>,
    scale: f32,
) -> impl Bundle {
    SpriteComponents {
        material: handle,
        transform: position_to_pixels(position),
        sprite: Sprite::new(Vec2::new(
            scale * TILE_SIZE as f32,
            scale * TILE_SIZE as f32,
        )),
        ..Default::default()
    }
}

fn update_positions(position: &Position, mut transform: Mut<'_, Transform>) {
    *transform = position_to_pixels(position);
}
