//! Lights and lighting.

use std::f32::consts::PI;

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use crate::{asset_management::palette::LIGHT_SUN, simulation::geometry::Height};

/// Handles all lighting logic
pub(super) struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: LIGHT_SUN,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_startup_system(spawn_sun);
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_sun(mut commands: Commands) {
    // The distance from the sun to the origin
    let sun_height = 2. * Height::MAX.into_world_pos();
    // The angle of the sun, relative to its position at noon
    // The sun is aligned such that -x is west, +x is east
    // This will change throughout the day, from - PI/2 to PI/2
    let sun_angle = PI / 4.;

    /// The angle that the sun is offset from vertical
    ///
    /// This technically changes with latitude.
    /// Should be 0 at the equator and
    const ELEVATION_ANGLE: f32 = 23.5 / 360. * PI / 2.;

    // Set the height
    let mut transform = Transform::from_xyz(0., sun_height, 0.);

    // Offset by the sun angle, then the elevation angle,
    transform.rotate_around(
        Vec3::ZERO,
        Quat::from_euler(EulerRot::XZY, sun_angle, ELEVATION_ANGLE, 0.),
    );

    // Look at the origin to point in the right direction
    transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: LIGHT_SUN,
            illuminance: 1.2e5,
            shadows_enabled: true,
            ..Default::default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 3,
            minimum_distance: 5.,
            maximum_distance: 200.,
            first_cascade_far_bound: 100.,
            overlap_proportion: 0.5,
        }
        .build(),
        transform,
        ..default()
    });
}
