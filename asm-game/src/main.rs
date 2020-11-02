use asm_lib::*;
use bevy::prelude::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(generation::GenerationPlugin)
        .add_plugin(graphics::GraphicsPlugin)
        .add_plugin(pheromones::PheromonesPlugin)
        .add_plugin(signals::SignalsPlugin)
        .add_plugin(structures::StructuresPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .add_plugin(units::UnitsPlugin)
        .run();
}