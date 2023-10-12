use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowPlugin};
use bevy_framepace::FramepacePlugin;
use emergence_lib::world_gen::GenerationConfig;

fn main() {
    App::new()
        .add_pluginss(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Emergence".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..Default::default()
        }))
        // This is turned on and off in the world gen state management code.
        .add_plugins(FramepacePlugin)
        .add_plugins(emergence_lib::asset_management::AssetManagementPlugin)
        .add_plugins(emergence_lib::simulation::SimulationPlugin {
            gen_config: GenerationConfig::standard(),
        })
        .add_plugins(emergence_lib::player_interaction::InteractionPlugin)
        .add_plugins(emergence_lib::graphics::GraphicsPlugin)
        .add_plugins(emergence_lib::ui::UiPlugin)
        .run();
}
