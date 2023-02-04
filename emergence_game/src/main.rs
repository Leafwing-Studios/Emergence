use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use emergence_lib::simulation::generation::GenerationConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Emergence".to_string(),
                // choose `AutoNoVsync` as it is more widely supported than `Immediate`
                present_mode: PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(emergence_lib::simulation::SimulationPlugin {
            gen_config: GenerationConfig::default(),
        })
        .add_plugin(emergence_lib::player_interaction::InteractionPlugin)
        .add_plugin(emergence_lib::graphics::GraphicsPlugin)
        .add_plugin(emergence_lib::asset_management::AssetManagementPlugin)
        .add_plugin(emergence_lib::dev_tools::DevToolsPlugin)
        .run();
}
