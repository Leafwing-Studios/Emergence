use bevy::prelude::*;

use crate::graphics::sprite_bundle_from_position;
use crate::position::Position;

#[derive(Clone, Default)]
pub struct Tile;

#[derive(Bundle, Default)]
pub struct TileBundle {
    tile: Tile,
    position: Position,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl TileBundle {
    pub fn new(position: Position, material: Handle<ColorMaterial>) -> Self {
        Self {
            sprite_bundle: sprite_bundle_from_position(position, material),
            ..Default::default()
        }
    }
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, _app: &mut AppBuilder) {}
}
