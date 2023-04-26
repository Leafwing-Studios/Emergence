//! Code for rendering water.

use bevy::prelude::*;

use crate::simulation::geometry::{hexagonal_column, MapGeometry};

pub(super) struct WaterRenderingPlugin;

impl Plugin for WaterRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_water_handles)
            .add_system(render_water);
    }
}

/// Stores handles used for water rendering.
#[derive(Resource)]
struct WaterHandles {
    /// The handle for the water material.
    material: Handle<StandardMaterial>,
    /// The handle for the water mesh.
    mesh: Handle<Mesh>,
}

/// Initializes handles used for water rendering.
fn init_water_handles(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::Rgba {
            red: 0.,
            green: 0.,
            blue: 0.7,
            alpha: 0.2,
        },
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let mesh = hexagonal_column(&map_geometry.layout, 1.0);
    let mesh_handle = meshes.add(mesh);
    commands.insert_resource(WaterHandles {
        material,
        mesh: mesh_handle,
    });
}

/// A marker component for an entity that visualizes the water level.
#[derive(Component)]
struct Water;

/// Renders surface water.
fn render_water(
    map_geometry: Res<MapGeometry>,
    water_handles: Res<WaterHandles>,
    water_query: Query<Entity, With<Water>>,
    mut commands: Commands,
) {
    // FIXME: don't use immediate mode for this
    for entity in water_query.iter() {
        commands.entity(entity).despawn();
    }

    for tile_pos in map_geometry.valid_tile_positions() {
        if let Some(water_height) = map_geometry.get_surface_water_height(tile_pos) {
            commands
                .spawn(PbrBundle {
                    mesh: water_handles.mesh.clone_weak(),
                    material: water_handles.material.clone_weak(),
                    transform: Transform {
                        translation: tile_pos.into_world_pos(&map_geometry),
                        scale: Vec3::new(1.0, water_height.into_world_pos(), 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Water);
        }
    }
}
