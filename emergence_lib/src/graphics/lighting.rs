//! Lights and lighting.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    geometry::Height,
    graphics::palette::lighting::{LIGHT_MOON, LIGHT_STARS, LIGHT_SUN},
};

/// Handles all lighting logic
pub(super) struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 0.2,
            color: LIGHT_STARS,
        })
        // Need to wait for the player camera to spawn
        .add_startup_system(spawn_celestial_bodies.in_base_set(StartupSet::PostStartup))
        .add_systems(Update, (animate_celestial_body_transform,));
    }
}

/// Controls the position and properties of the sun and moon
#[derive(Component, Debug)]
pub(crate) struct CelestialBody {
    /// The distance from the origin of the directional light
    ///
    /// This has no effect on the angle: it simply needs to be high enough to avoid clipping at max world height.
    height: f32,
    /// The angle of the celestial body in radians, relative to the zenith (the sun's position at "noon").
    /// This will change throughout the day, from - PI/2 at dawn to PI/2 at dusk.
    pub(crate) hour_angle: f32,
    /// The angle that the sun is offset from the zenith in radians.
    ///
    /// In real life changes with latitude and season.
    declination: f32,
    /// The rotation around the y axis in radians.
    ///
    /// A value of 0.0 corresponds to east -> west travel.
    travel_axis: f32,
}

impl CelestialBody {
    /// The default angle that the sun is offset from the zenith in radians.
    const HOUR_ANGLE_AT_NOON: f32 = 23.5 / 360.;

    /// The starting settings for the sun
    fn sun() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: CelestialBody::HOUR_ANGLE_AT_NOON,
            declination: -PI / 4.,
            travel_axis: 0.,
        }
    }

    /// The starting settings for the moon
    fn moon() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: 0.,
            declination: CelestialBody::HOUR_ANGLE_AT_NOON,
            travel_axis: PI / 6.,
        }
    }
}

/// This component signals that this Entity is the primary celestial body for lighting.
#[derive(Component, Debug)]
pub(crate) struct Sun;

/// This component signals that this Entity is the secondary celestial body for lighting.
#[derive(Component, Debug)]
pub(crate) struct Moon;

/// Spawns a directional light source to illuminate the scene
fn spawn_celestial_bodies(mut commands: Commands) {
    let sun = CelestialBody::sun();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_SUN,
                illuminance: 8e4,
                shadows_enabled: false,
                ..Default::default()
            },
            ..default()
        })
        .insert(sun)
        .insert(Sun);

    let moon = CelestialBody::moon();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_MOON,
                illuminance: 3e4,
                shadows_enabled: false,
                ..Default::default()
            },
            ..default()
        })
        .insert(moon)
        .insert(Moon);
}

/// Moves celestial bodies to the correct position and orientation
// PERF: this doesn't need to run constantly if we're not moving the sun and moon
fn animate_celestial_body_transform(
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
                celestial_body.hour_angle,
                celestial_body.declination,
                celestial_body.travel_axis,
            ),
        );

        // Look at the origin to point in the right direction
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
