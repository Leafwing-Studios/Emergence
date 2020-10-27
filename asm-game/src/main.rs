use asm_lib::hello;
use bevy::prelude::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(hello::HelloPlugin)
        .run();
}
