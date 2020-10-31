use bevy::prelude::*;
use crate::config::MAP_SIZE;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(generate_terrain.system());
    }
}

fn generate_terrain() {
    dbg!(MAP_SIZE);
}
