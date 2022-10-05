use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Emergence".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(emergence_lib::generation::GenerationPlugin)
        .add_plugin(emergence_lib::diffusion::DiffusionPlugin)
        .run();
}
