use bevy::prelude::*;

use crate::config::MAP_DIAMETER;
use crate::graphics::make_sprite_components;
use crate::id::{Contents, ID};
use crate::position::Position;

pub struct Tile {}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Contents::from_elem((MAP_DIAMETER, MAP_DIAMETER), None))
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
    *contents = contents.map(|_| None);

    for (position, &id) in tile_query.iter() {
        contents[position] = Some(id);
    }
}
