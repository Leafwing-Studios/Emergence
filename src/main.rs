use bevy::prelude::*;

pub mod hello;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(hello::HelloPlugin)
        .run();
}
