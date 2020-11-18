use bevy::prelude::*;
use ndarray::prelude::*;

use crate::config::MAP_SIZE;
use crate::graphics::make_sprite_components;
use crate::utils::{Position, ID};

pub struct Tile {}

#[derive(Debug, Clone)]
struct Contents {
    id: Array2<ID>,
}

const MAP_DIAMETER: usize = (2 * MAP_SIZE + 1) as usize;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Contents {
            id: Array::from_elem((MAP_DIAMETER, MAP_DIAMETER).f(), ID::Nothing),
        })
        .add_stage_after(stage::UPDATE, "BOOKKEEPING")
        .add_system_to_stage("BOOKKEEPING", update_contents.system());
    }
}

pub fn build_tile(commands: &mut Commands, handle: Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Tile {})
        .with(position);
}

fn update_contents(
    mut tile_query: Query<(&Position, &ID), (Changed<Position>, Without<Tile>)>,
    mut contents: ResMut<Contents>,
) {
}
