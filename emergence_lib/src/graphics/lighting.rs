//! Lights and lighting.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    graphics::palette::lighting::{LIGHT_MOON, LIGHT_STARS, LIGHT_SUN},
    simulation::{geometry::Height, light::Illuminance},
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
        // FIXME: shadows are blocked on better rendering performance.
        // Tracked in https://github.com/Leafwing-Studios/Emergence/issues/726
        //.insert_resource(DirectionalLightShadowMap { size: 8192 })
        // Need to wait for the player camera to spawn
        .add_startup_system(spawn_celestial_bodies.in_base_set(StartupSet::PostStartup))
        .add_systems((
            animate_celestial_body_transform,
            animate_celestial_body_brightness,
        ));
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
    /// The base illuminance of the [`DirectionalLight`]
    illuminance: f32,
    /// The total effect of temporary modifiers to the [`DirectionalLight`]'s illuminance.
    ///
    /// This defaults to 1.0, and is multiplied by the base illuminance.
    light_level: f32,
    /// The number of in-game days required to complete a full cycle.
    pub(crate) days_per_cycle: f32,
}

impl CelestialBody {
    /// The default angle that the sun is offset from the zenith in radians.
    const DEFAULT_NOON_RADIANS: f32 = 23.5 / 360.;

    /// Sets the `light_level` of this celestial body.
    pub(crate) fn set_light_level(&mut self, light_level: f32) {
        self.light_level = light_level;
    }

    /// Computes the total irradiance produced by this celestial body based on its position in the sky.
    pub(crate) fn compute_light(&self) -> Illuminance {
        CelestialBody::compute_illuminance(
            self.light_level,
            self.hour_angle,
            self.declination,
            self.illuminance,
        )
    }

    /// Computes the maximum total irradiance produced by this celestial body based on its brightest possible position in the sky.
    ///
    /// This is used to determine the maximum brightness of the directional light.
    pub(crate) fn compute_max_light(&self) -> Illuminance {
        CelestialBody::compute_illuminance(
            1.0,
            CelestialBody::DEFAULT_NOON_RADIANS,
            self.declination,
            self.illuminance,
        )
    }

    /// Computes the total irradiance produced by a celestial body given `hour_angle`, `declination`, and `illuminance`.
    fn compute_illuminance(
        light_level: f32,
        hour_angle: f32,
        declination: f32,
        illuminance: f32,
    ) -> Illuminance {
        // Computes the total angle formed by the celestial body and the horizon
        //
        // We cannot simply use the progress, as the inclination also needs to be taken into account.
        // See https://en.wikipedia.org/wiki/Solar_zenith_angle
        // We're treating the latitude here as equatorial.
        let cos_solar_zenith_angle = hour_angle.cos() * declination.cos();
        let solar_zenith_angle = cos_solar_zenith_angle.acos();
        Illuminance(light_level * illuminance * solar_zenith_angle.cos().max(0.))
    }

    /// The starting settings for the sun
    fn sun() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: -PI / 4.,
            declination: CelestialBody::DEFAULT_NOON_RADIANS * PI / 2.,
            travel_axis: 0.,
            illuminance: 8e4,
            light_level: 1.0,
            days_per_cycle: 1.0,
        }
    }

    /// The starting settings for the moon
    fn moon() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: 0.,
            declination: CelestialBody::DEFAULT_NOON_RADIANS * PI / 2.,
            travel_axis: PI / 6.,
            illuminance: 3e4,
            light_level: 1.0,
            days_per_cycle: 29.53,
        }
    }
}

/// This component signals that this Entity is the primary celestial body for lighting.
#[derive(Component, Debug)]
pub(crate) struct PrimaryCelestialBody;

/// Spawns a directional light source to illuminate the scene
#[allow(dead_code)]
fn spawn_celestial_bodies(mut commands: Commands) {
    /*  Shadows are currently disabled for perf reasons:
    Tracked in https://github.com/Leafwing-Studios/Emergence/issues/726

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
    */

    let sun = CelestialBody::sun();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_SUN,
                illuminance: sun.illuminance,
                shadows_enabled: false,
                ..Default::default()
            },
            ..default()
        })
        .insert(sun)
        .insert(PrimaryCelestialBody);

    let moon = CelestialBody::moon();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_MOON,
                illuminance: moon.illuminance,
                shadows_enabled: false,
                ..Default::default()
            },
            ..default()
        })
        .insert(moon);
}

/// Moves celestial bodies to the correct position and orientation
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

/// Adjusts the brightness of celestial bodies based on their position in the sky
fn animate_celestial_body_brightness(
    mut query: Query<(&CelestialBody, &mut DirectionalLight), Changed<CelestialBody>>,
) {
    for (celestial_body, mut directional_light) in query.iter_mut() {
        let current_illuminance = celestial_body.compute_light();
        directional_light.illuminance = current_illuminance.0;
    }
}
