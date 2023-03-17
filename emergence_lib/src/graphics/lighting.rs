//! Lights and lighting.

use std::f32::consts::PI;

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use crate::{
    asset_management::palette::LIGHT_SUN, player_interaction::camera::CameraSettings,
    simulation::geometry::Height,
};

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
        // Need to wait for the player camera to spawn
        .add_startup_system(spawn_sun.in_base_set(StartupSet::PostStartup));
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_sun(mut commands: Commands, camera_query: Query<&CameraSettings>) {
    let camera_settings = camera_query.single();

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
            illuminance: 1e5,
            shadows_enabled: true,
            ..Default::default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            // Max is 4, as of Bevy 0.10
            num_cascades: 4,
            // Shadows must be visible even when fully zoomed in
            minimum_distance: camera_settings.min_zoom,
            // Shadows must be visible even when fully zoomed out
            maximum_distance: 4. * camera_settings.max_zoom,
            first_cascade_far_bound: 2. * camera_settings.min_zoom,
            overlap_proportion: 0.3,
        }
        .build(),
        transform,
        ..default()
    });
}
