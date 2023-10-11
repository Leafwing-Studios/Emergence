//! Code for rendering water.

use bevy::prelude::*;

use crate::{
    geometry::{hexagonal_column, DiscreteHeight, Height, MapGeometry, VoxelPos},
    water::{ocean::Ocean, WaterConfig, WaterDepth},
};

use super::{palette::environment::WATER, GraphicsSet};

/// A plugin that controls how water is displayed.
pub(super) struct WaterRenderingPlugin;

impl Plugin for WaterRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_water_handles)
            .add_systems(Update, render_water.in_set(GraphicsSet));
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: WATER,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let mesh = hexagonal_column(1.0);
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
    water_handles: Res<WaterHandles>,
    rendered_water_query: Query<Entity, With<Water>>,
    water_config: Res<WaterConfig>,
    water_depth_query: Query<(&VoxelPos, &WaterDepth)>,
    map_geometry: Res<MapGeometry>,
    ocean: Res<Ocean>,
    mut commands: Commands,
) {
    // FIXME: don't use immediate mode for this
    for entity in rendered_water_query.iter() {
        commands.entity(entity).despawn();
    }

    for (voxel_pos, water_depth) in water_depth_query.iter() {
        let surface_water_depth = water_depth.surface_water_depth();

        if surface_water_depth > Height::ZERO {
            commands
                .spawn(PbrBundle {
                    mesh: water_handles.mesh.clone_weak(),
                    material: water_handles.material.clone_weak(),
                    transform: Transform {
                        translation: voxel_pos.top_of_tile(),
                        scale: Vec3::new(1.0, surface_water_depth.into_world_pos(), 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Water);
        }
    }

    if water_config.enable_oceans {
        for hex in map_geometry.ocean_tiles() {
            let voxel_pos = VoxelPos {
                hex,
                height: DiscreteHeight::ZERO,
            };

            commands
                .spawn(PbrBundle {
                    mesh: water_handles.mesh.clone_weak(),
                    material: water_handles.material.clone_weak(),
                    transform: Transform {
                        translation: voxel_pos.top_of_tile(),
                        scale: Vec3::new(1.0, ocean.height().into_world_pos(), 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Water);
        }
    }
}
