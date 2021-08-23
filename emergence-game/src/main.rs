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
        .run();
}
