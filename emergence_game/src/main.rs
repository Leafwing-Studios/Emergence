use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Emergence".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(emergence_lib::graphics::GraphicsPlugin)
        .add_plugin(emergence_lib::SimulationPlugin)
        .add_plugin(emergence_lib::InteractionPlugin)
        .run();
}
