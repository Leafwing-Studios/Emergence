use bevy::{prelude::*, window::WindowMode};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Emergence".to_string(),
            vsync: true,
            resizable: false,
            mode: WindowMode::Fullscreen { use_size: false },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(emergence_lib::tilemap::TilemapPlugin)
        .run();
}
