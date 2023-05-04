//! Code for rendering water.

use bevy::prelude::*;

use crate::{
    simulation::geometry::{hexagonal_column, Height, MapGeometry},
    water::{WaterConfig, WaterTable},
};

use super::palette::environment::WATER;

/// A plugin that controls how water is displayed.
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
        base_color: WATER,
        unlit: true,
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
    water_table: Res<WaterTable>,
    water_handles: Res<WaterHandles>,
    water_query: Query<Entity, With<Water>>,
    water_config: Res<WaterConfig>,
    mut commands: Commands,
) {
    // FIXME: don't use immediate mode for this
    for entity in water_query.iter() {
        commands.entity(entity).despawn();
    }

    for tile_pos in map_geometry.valid_tile_positions() {
        let surface_water_depth = water_table.surface_water_depth(tile_pos);

        if surface_water_depth > Height::ZERO {
            commands
                .spawn(PbrBundle {
                    mesh: water_handles.mesh.clone_weak(),
                    material: water_handles.material.clone_weak(),
                    transform: Transform {
                        translation: tile_pos.top_of_tile(&map_geometry),
                        scale: Vec3::new(1.0, surface_water_depth.into_world_pos(), 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Water);
        }
    }

    if water_config.enable_oceans {
        for tile_pos in map_geometry.ocean_tiles() {
            commands
                .spawn(PbrBundle {
                    mesh: water_handles.mesh.clone_weak(),
                    material: water_handles.material.clone_weak(),
                    transform: Transform {
                        translation: tile_pos.top_of_tile(&map_geometry),
                        scale: Vec3::new(1.0, water_table.ocean_height().into_world_pos(), 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Water);
        }
    }
}
