//! Rendering and animation logic.

use bevy::prelude::*;
use bevy_toon_shader::{ToonShaderMaterial, ToonShaderPlugin};

use crate::asset_management::AssetState;

use self::{
    atmosphere::AtmospherePlugin,
    lighting::LightingPlugin,
    palette::lighting::{LIGHT_STARS, LIGHT_SUN},
    structures::remove_ghostly_shadows,
    terrain::manage_litter_piles,
};

mod atmosphere;
pub(crate) mod lighting;
pub(crate) mod palette;
mod structures;
mod terrain;
mod units;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ToonShaderPlugin)
            .add_plugin(LightingPlugin)
            .add_plugin(AtmospherePlugin)
            .add_system(manage_litter_piles.run_if(in_state(AssetState::FullyLoaded)))
            // Run these after Update to avoid panics due to despawned entities
            .add_systems(
                (inherit_materials, remove_ghostly_shadows).in_base_set(CoreSet::PostUpdate),
            )
            .add_startup_system(spawn_debug_toon_shaded_cube);
    }
}

/// A material that will be inherited by all children in the scene.
#[derive(Component, Debug, Deref)]
pub(crate) struct InheritedMaterial(pub(crate) Handle<StandardMaterial>);

/// Applies [`InheritedMaterial`] to all child entities recursively.
pub(super) fn inherit_materials(
    root_structure_query: Query<(Entity, &InheritedMaterial)>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
) {
    for (root_entity, inherited_material) in root_structure_query.iter() {
        for child in children.iter_descendants(root_entity) {
            if let Ok(mut child_material) = material_query.get_mut(child) {
                *child_material = inherited_material.clone_weak();
            }
        }
    }
}

fn spawn_debug_toon_shaded_cube(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<ToonShaderMaterial>>,
) {
    let mesh = Mesh::from(shape::Cube { size: 80.0 });
    let material = ToonShaderMaterial {
        color: Color::rgb(0.5, 0.5, 0.5),
        sun_color: LIGHT_SUN,
        ambient_color: LIGHT_STARS,
        // Automatically updated
        sun_dir: Vec3::default(),
        // Automatically updated
        camera_pos: Vec3::default(),
        base_color_texture: None,
    };

    // Store the generated assets and get a handle to them
    let mesh = mesh_assets.add(mesh);
    let material = material_assets.add(material);

    commands.spawn(MaterialMeshBundle {
        mesh,
        material,
        transform: Transform::default(),
        ..Default::default()
    });
}
