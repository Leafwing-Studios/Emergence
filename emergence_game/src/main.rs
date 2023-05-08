use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowPlugin};
use bevy_framepace::FramepacePlugin;
use emergence_lib::world_gen::GenerationConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Emergence".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..Default::default()
        }))
        // This is turned on and off in the world gen state management code.
        .add_plugin(FramepacePlugin)
        .add_plugin(emergence_lib::simulation::GeometryPlugin {
            gen_config: GenerationConfig::default(),
        })
        .add_plugin(emergence_lib::asset_management::AssetManagementPlugin)
        .add_plugin(emergence_lib::simulation::SimulationPlugin {
            gen_config: GenerationConfig::default(),
        })
        .add_plugin(emergence_lib::player_interaction::InteractionPlugin)
        .add_plugin(emergence_lib::graphics::GraphicsPlugin)
        .add_plugin(emergence_lib::ui::UiPlugin)
        .run();
}
