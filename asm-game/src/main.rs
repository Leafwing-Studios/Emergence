use asm_lib::*;
use bevy::prelude::*;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Assimilation".to_string(),
            width: config::WINDOW_WIDTH,
            height: config::WINDOW_HEIGHT,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(generation::GenerationPlugin)
        .add_plugin(graphics::GraphicsPlugin)
        .add_plugin(pheromones::PheromonesPlugin)
        .add_plugin(signals::SignalsPlugin)
        .add_plugin(structures::StructuresPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .add_plugin(units::UnitsPlugin)
        .run();
}
