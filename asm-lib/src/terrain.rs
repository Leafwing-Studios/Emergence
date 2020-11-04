use bevy::prelude::*;

use crate::config::TILE_SIZE;
use crate::utils::Position;

pub struct Tile {}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app;
    }
}
