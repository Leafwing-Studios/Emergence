
use bevy::prelude::*;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(setup_graphics.system())
        .add_system(render.system());

    }
}

fn setup_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let _materials =  asset_server.load_folder("");

    commands
        .spawn(Camera2dComponents::default());
}

fn render(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    let clover = asset_server.get_handle("clover.png");

    commands
    .spawn(SpriteComponents {
        // TODO: This seems incredibly unidiomatic and doesn't match the AssetServer example
        material: materials.add(clover.into()),
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
        sprite: Sprite::new(Vec2::new(500.0, 500.0)),
        ..Default::default()
    });
}