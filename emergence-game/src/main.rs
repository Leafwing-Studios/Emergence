use bevy::prelude::*;
use emergence_lib::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Emergence".to_string(),
            width: config::WINDOW_WIDTH,
            height: config::WINDOW_HEIGHT,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(entity_map::EntityMapPlugin)
        .add_plugin(generation::GenerationPlugin)
        .add_plugin(graphics::GraphicsPlugin)
        .add_plugin(pheromones::PheromonesPlugin)
        .add_plugin(signals::SignalsPlugin)
        .add_plugin(structures::StructuresPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .add_plugin(units::UnitsPlugin)
        .add_plugin(ui::UiPlugin)
        .run();
}
