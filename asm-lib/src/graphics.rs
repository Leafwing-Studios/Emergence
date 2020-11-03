use bevy::prelude::*;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system())
            .add_system(render.system());
    }
}

pub struct TileMaterial(Handle<ColorMaterial>);
pub struct AntMaterial(Handle<ColorMaterial>);
pub struct PlantMaterial(Handle<ColorMaterial>);
pub struct FungiMaterial(Handle<ColorMaterial>);

fn setup_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_handle = asset_server.load("path-tile.png");
    let ant_handle = asset_server.load("ant.png");
    let plant_handle = asset_server.load("clover.png");
    let fungi_handle = asset_server.load("mushroom-gills.png");

    commands
        .spawn(Camera2dComponents::default())
        .insert_resource(TileMaterial(materials.add(tile_handle.into())))
        .insert_resource(AntMaterial(materials.add(ant_handle.into())))
        .insert_resource(PlantMaterial(materials.add(plant_handle.into())))
        .insert_resource(FungiMaterial(materials.add(fungi_handle.into())));
}

fn render(mut commands: Commands, plant_material: Res<PlantMaterial>) {
    commands.spawn(SpriteComponents {
        material: plant_material.0.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
        sprite: Sprite::new(Vec2::new(500.0, 500.0)),
        ..Default::default()
    });
}
