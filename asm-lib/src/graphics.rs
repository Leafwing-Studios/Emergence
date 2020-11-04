use bevy::prelude::*;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system());
    }
}

fn setup_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let _assets = asset_server.load_folder("");

    commands.spawn(Camera2dComponents::default());
}
