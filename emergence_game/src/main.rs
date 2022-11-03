use bevy::prelude::*;
use emergence_lib::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Emergence".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(cursor::CursorTilePosPlugin)
        .add_plugin(hive_mind::HiveMindPlugin)
        .add_plugin(terrain::generation::GenerationPlugin)
        .add_plugin(organisms::structures::StructuresPlugin)
        .add_plugin(organisms::units::UnitsPlugin)
        .add_plugin(signals::SignalsPlugin)
        .run();
}
