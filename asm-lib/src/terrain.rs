use bevy::prelude::*;

use crate::graphics::make_sprite_components;
use crate::utils::{Position, ID};

pub struct Tile {}
#[derive(Debug, Clone, Copy)]
pub struct Contents {
    contents: Option<ID>,
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        //app.add_system(update_contents.system());
    }
}

pub fn build_tile(commands: &mut Commands, handle: Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Tile {})
        .with(position)
        .with(Contents { contents: None });
}
/*
fn update_contents(
    mut tile_query: Query<(&Position, &Contents)>,
    stuff_query: Query<Changed<&Position>>,
) {
} */
