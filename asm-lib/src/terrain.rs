use bevy::prelude::*;

use crate::graphics::make_sprite_components;
use crate::utils::Position;

pub struct Tile {}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app;
    }
}

pub fn build_tile(commands: &mut Commands, handle: Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Tile {})
        .with(position);
}
