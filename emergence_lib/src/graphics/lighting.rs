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
        .add_startup_system(spawn_sun.in_base_set(StartupSet::PostStartup))
        .add_system(set_celestial_body_transform);
    }
}

/// Controls the position of the sun
#[derive(Component, Debug)]
pub(crate) struct CelestialBody {
    /// The distance from the origin of the directional light
    ///
    /// This has no effect on the angle: it simply needs to be high enough to avoid clipping at max world height.
    height: f32,
    /// The angle of the sun in radians, relative to its position at noon
    /// The sun is aligned such that -x is west, +x is east
    /// This will change throughout the day, from - PI/2 to PI/2
    pub(crate) progress: f32,
    /// The angle that the sun is offset from vertical in radians
    ///
    /// This technically changes with latitude.
    /// Should be zero from the equator.
    inclination: f32,
}

impl Default for CelestialBody {
    fn default() -> Self {
        Self {
            height: 2. * Height::MAX.into_world_pos(),
            progress: -PI / 4.,
            inclination: 23.5 / 360. * PI / 2.,
        }
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_sun(mut commands: Commands, camera_query: Query<&CameraSettings>) {
    let camera_settings = camera_query.single();
    commands
        .spawn(DirectionalLightBundle {
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
            ..default()
        })
        .insert(CelestialBody::default());
}

/// Moves celestial bodies to the correct position and orientation
fn set_celestial_body_transform(
    mut query: Query<(&mut Transform, &CelestialBody), Changed<CelestialBody>>,
) {
    for (mut transform, celestial_body) in query.iter_mut() {
        // Set the height
        *transform = Transform::from_xyz(0., celestial_body.height, 0.);

        // Offset by the sun angle, then the elevation angle,
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(
                EulerRot::XZY,
                celestial_body.progress,
                celestial_body.inclination,
                0.,
            ),
        );

        // Look at the origin to point in the right direction
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
