use bevy::prelude::*;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_graphics.system())
            .add_system(render.system());
    }
}

trait MaterialHandle {
    fn new(h: Handle<ColorMaterial>) -> Self;
}

macro_rules! material {
    ($name:ident) => {
        pub struct $name(Handle<ColorMaterial>);
        impl MaterialHandle for $name {
            fn new(h: Handle<ColorMaterial>) -> Self {
                Self(h)
            }
        }
    };
}

material!(TileMaterial);
material!(PlantMaterial);
material!(FungiMaterial);
material!(AntMaterial);

fn insert_material<T: MaterialHandle + Resource>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    path: &str,
) {
    let handle = asset_server.load(path);
    commands.insert_resource(T::new(materials.add(handle.into())));
}

fn setup_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dComponents::default());

    let (commands, asset_server, materials) = (&mut commands, &asset_server, &mut materials);
    insert_material::<TileMaterial>(commands, asset_server, materials, "path-tile.png");
    insert_material::<AntMaterial>(commands, asset_server, materials, "ant-tile.png");
    insert_material::<PlantMaterial>(commands, asset_server, materials, "plant-tile.png");
    insert_material::<FungiMaterial>(commands, asset_server, materials, "fungi-tile.png");
}

fn render(mut commands: Commands, plant_material: Res<PlantMaterial>) {
    commands.spawn(SpriteComponents {
        material: plant_material.0.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
        sprite: Sprite::new(Vec2::new(500.0, 500.0)),
        ..Default::default()
    });
}
