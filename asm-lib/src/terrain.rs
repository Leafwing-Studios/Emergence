use bevy::prelude::*;
use ndarray::prelude::*;

use crate::config::MAP_DIAMETER;
use crate::graphics::make_sprite_components;
use crate::utils::{Position, ID};

pub struct Tile {}

#[derive(Debug, Clone)]
pub struct Contents {
    pub id: Array2<ID>,
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Contents {
            id: Array::from_elem((MAP_DIAMETER, MAP_DIAMETER), ID::Nothing),
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

fn update_contents(tile_query: Query<(&Position, &ID)>, mut contents: ResMut<Contents>) {
    contents.id = contents.id.map(|_| ID::Nothing);

    for (position, id) in tile_query.iter() {
        contents.id[position.to_array_ind()] = *id;
    }
}
