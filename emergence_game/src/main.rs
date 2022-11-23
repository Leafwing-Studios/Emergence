use bevy::prelude::*;
use bevy::window::WindowPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Emergence".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(emergence_lib::simulation::SimulationPlugin)
        .add_plugin(emergence_lib::InteractionPlugin)
        .add_plugin(emergence_lib::graphics::GraphicsPlugin)
        .run();
}
