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
        // Controls the resolution of shadows cast by the sun
        .insert_resource(DirectionalLightShadowMap { size: 8192 })
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
            // We don't really want to vary shadow quality with distance from the light sourece, since it's always basically constant
            num_cascades: 1,
            // At max map height, this should be about the correct distance from the sun
            minimum_distance: sun_height / 2.,
            // Heights are never 0, so this is a good settings
            maximum_distance: sun_height,
            // We want to be some small multiple of the minimum distance
            first_cascade_far_bound: 1.2 * (sun_height / 2.),
            overlap_proportion: 0.8,
        }
        .build(),
        transform,
        ..default()
    });
}
