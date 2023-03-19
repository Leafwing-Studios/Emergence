//! Lights and lighting.

use std::f32::consts::PI;

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use crate::{
    asset_management::palette::{LIGHT_MOON, LIGHT_STARS, LIGHT_SUN},
    player_interaction::camera::CameraSettings,
    simulation::geometry::Height,
};

/// Handles all lighting logic
pub(super) struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 0.2,
            color: LIGHT_STARS,
        })
        // Controls the resolution of shadows cast by the sun
        .insert_resource(DirectionalLightShadowMap { size: 8192 })
        // Need to wait for the player camera to spawn
        .add_startup_system(spawn_celestial_bodies.in_base_set(StartupSet::PostStartup))
        .add_system(set_celestial_body_transform)
        .add_system(toggle_lights_based_on_ground_clipping);
    }
}

/// Controls the position of the sun
#[derive(Component, Debug)]
pub(crate) struct CelestialBody {
    /// The distance from the origin of the directional light
    ///
    /// This has no effect on the angle: it simply needs to be high enough to avoid clipping at max world height.
    height: f32,
    /// The angle of the celestial body in radians, relative to its position at "noon".
    /// This will change throughout the day, from - PI/2 at dawn to PI/2 at dusk.
    pub(crate) progress: f32,
    /// The angle that the sun is offset from vertical in radians
    ///
    /// This technically changes with latitude.
    /// Should be zero from the equator.
    inclination: f32,
    /// The rotation around the y axis in radians.
    ///
    /// A value of 0.0 corresponds to east -> west travel.
    travel_axis: f32,
    /// The number of in-game days required to complete a full cycle.
    pub(crate) days_per_cycle: f32,
}

impl CelestialBody {
    /// The starting settings for the sun
    fn sun() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            progress: -PI / 4.,
            inclination: 23.5 / 360. * PI / 2.,
            travel_axis: 0.,
            days_per_cycle: 1.0,
        }
    }

    /// The starting settings for the moon
    fn moon() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            progress: 0.,
            inclination: 23.5 / 360. * PI / 2.,
            travel_axis: PI / 6.,
            days_per_cycle: 2.0,
        }
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_celestial_bodies(mut commands: Commands, camera_query: Query<&CameraSettings>) {
    let camera_settings = camera_query.single();
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        // Max is 4, as of Bevy 0.10
        num_cascades: 4,
        // Shadows must be visible even when fully zoomed in
        minimum_distance: camera_settings.min_zoom,
        // Shadows must be visible even when fully zoomed out
        maximum_distance: 4. * camera_settings.max_zoom,
        first_cascade_far_bound: 2. * camera_settings.min_zoom,
        overlap_proportion: 0.3,
    }
    .build();

    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_SUN,
                illuminance: 8e4,
                shadows_enabled: true,
                ..Default::default()
            },
            cascade_shadow_config: cascade_shadow_config.clone(),
            ..default()
        })
        .insert(CelestialBody::sun());
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
                EulerRot::YXZ,
                celestial_body.travel_axis,
                celestial_body.progress,
                celestial_body.inclination,
            ),
        );

        // Look at the origin to point in the right direction
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Disables lights that are under the ground
fn toggle_lights_based_on_ground_clipping(
    mut query: Query<(&CelestialBody, &mut Visibility, &mut DirectionalLight)>,
) {
    for (celestial_body, mut visibility, mut directional_light) in query.iter_mut() {
        if celestial_body.progress > 0.5 {
            *visibility = Visibility::Hidden;
            directional_light.shadows_enabled = false;
        } else {
            *visibility = Visibility::Visible;
            directional_light.shadows_enabled = true;
        }
    }
}
