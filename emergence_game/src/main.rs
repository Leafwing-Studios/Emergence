use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use emergence_lib::simulation::generation::GenerationConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Emergence".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..Default::default()
        }))
        .add_plugin(emergence_lib::simulation::GeometryPlugin {
            gen_config: GenerationConfig::default(),
        })
        .add_plugin(emergence_lib::asset_management::AssetManagementPlugin)
        .add_plugin(emergence_lib::simulation::SimulationPlugin {
            gen_config: GenerationConfig::default(),
        })
        .add_plugin(emergence_lib::player_interaction::InteractionPlugin)
        .add_plugin(emergence_lib::graphics::GraphicsPlugin)
        .add_plugin(emergence_lib::infovis::InfoVisPlugin)
        .add_plugin(emergence_lib::ui::UiPlugin)
        .run();
}
