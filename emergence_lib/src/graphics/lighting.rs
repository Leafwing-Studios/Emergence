//! Lights and lighting.

use std::f32::consts::PI;

use bevy::{
    math::{Affine3A, Vec3A},
    pbr::{
        update_directional_light_cascades, CascadeShadowConfig, Cascades, DirectionalLightShadowMap,
    },
    prelude::*,
    render::primitives::Aabb,
};

use crate::{
    asset_management::palette::lighting::{LIGHT_MOON, LIGHT_STARS, LIGHT_SUN},
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
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        // Need to wait for the player camera to spawn
        .add_startup_system(spawn_celestial_bodies.in_base_set(StartupSet::PostStartup))
        .add_system(set_celestial_body_transform)
        .add_system(
            hack_cascades
                .in_base_set(CoreSet::PostUpdate)
                .after(update_directional_light_cascades),
        );
    }
}

/// Controls the position of the sun
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
    /// The number of in-game days required to complete a full cycle.
    pub(crate) days_per_cycle: f32,
}

impl CelestialBody {
    /// Computes the total irradiance produced by this celestial body based on its position in the sky.
    pub(crate) fn compute_light(&self) -> Illuminance {
        // Computes the total angle formed by the celestial body and the horizon
        //
        // We cannot simply use the progress, as the inclination also needs to be taken into account.
        // See https://en.wikipedia.org/wiki/Solar_zenith_angle
        // We're treating the latitude here as equatorial.
        let cos_solar_zenith_angle = self.hour_angle.cos() * self.declination.cos();
        let solar_zenith_angle = cos_solar_zenith_angle.acos();
        Illuminance(self.illuminance * solar_zenith_angle.cos().max(0.))
    }

    /// The starting settings for the sun
    fn sun() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: -PI / 4.,
            declination: 23.5 / 360. * PI / 2.,
            travel_axis: 0.,
            illuminance: 8e4,
            days_per_cycle: 1.0,
        }
    }

    /// The starting settings for the moon
    fn moon() -> CelestialBody {
        CelestialBody {
            height: 2. * Height::MAX.into_world_pos(),
            hour_angle: 0.,
            declination: 23.5 / 360. * PI / 2.,
            travel_axis: PI / 6.,
            illuminance: 1e4,
            days_per_cycle: 29.53,
        }
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_celestial_bodies(mut commands: Commands) {
    let cascade_shadow_config = CascadeShadowConfig {
        bounds: vec![0.0],
        overlap_proportion: 0.3,
        minimum_distance: 0.1,
    };

    let sun = CelestialBody::sun();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_SUN,
                illuminance: sun.illuminance,
                shadows_enabled: true,
                ..Default::default()
            },
            cascade_shadow_config: cascade_shadow_config.clone(),
            ..default()
        })
        .insert(sun);

    let moon = CelestialBody::moon();
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_MOON,
                illuminance: moon.illuminance,
                shadows_enabled: true,
                ..Default::default()
            },
            cascade_shadow_config,
            ..default()
        })
        .insert(moon);
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
                celestial_body.hour_angle,
                celestial_body.declination,
                celestial_body.travel_axis,
            ),
        );

        // Look at the origin to point in the right direction
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

#[inline]
fn aabb_corners(aabb: &Aabb) -> [Vec3A; 8] {
    [
        aabb.center
            + aabb.half_extents.z * Vec3A::Z
            + aabb.half_extents.y * Vec3A::Y
            + aabb.half_extents.x * Vec3A::X,
        aabb.center + aabb.half_extents.z * Vec3A::Z + aabb.half_extents.y * Vec3A::Y
            - aabb.half_extents.x * Vec3A::X,
        aabb.center + aabb.half_extents.z * Vec3A::Z - aabb.half_extents.y * Vec3A::Y
            + aabb.half_extents.x * Vec3A::X,
        aabb.center + aabb.half_extents.z * Vec3A::Z
            - aabb.half_extents.y * Vec3A::Y
            - aabb.half_extents.x * Vec3A::X,
        aabb.center - aabb.half_extents.z * Vec3A::Z
            + aabb.half_extents.y * Vec3A::Y
            + aabb.half_extents.x * Vec3A::X,
        aabb.center - aabb.half_extents.z * Vec3A::Z + aabb.half_extents.y * Vec3A::Y
            - aabb.half_extents.x * Vec3A::X,
        aabb.center - aabb.half_extents.z * Vec3A::Z - aabb.half_extents.y * Vec3A::Y
            + aabb.half_extents.x * Vec3A::X,
        aabb.center
            - aabb.half_extents.z * Vec3A::Z
            - aabb.half_extents.y * Vec3A::Y
            - aabb.half_extents.x * Vec3A::X,
    ]
}

#[inline]
fn aabb_to_transformed_min_max(aabb: &Aabb, transform: &Affine3A) -> (Vec3A, Vec3A) {
    aabb_corners(aabb)
        .into_iter()
        .map(|corner| transform.transform_point3a(corner))
        .fold(
            (Vec3A::splat(f32::MAX), Vec3A::splat(f32::MIN)),
            |(min, max), corner| (min.min(corner), max.max(corner)),
        )
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub fn hack_cascades(
    directional_light_shadow_map: Res<DirectionalLightShadowMap>,
    views: Query<(Entity, &GlobalTransform, &Camera)>,
    meshes: Query<(&GlobalTransform, &Aabb)>, // (With<Handle<Mesh>>, Without<Camera>, Without<DirectionalLight>)>,
    mut lights: Query<(
        &GlobalTransform,
        &DirectionalLight,
        &mut CascadeShadowConfig,
        &mut Cascades,
    )>,
) {
    let (mut min, mut max) = (Vec3A::splat(f32::MAX), Vec3A::splat(f32::MIN));
    for (transform, aabb) in &meshes {
        let (mn, mx) = aabb_to_transformed_min_max(aabb, &transform.affine());
        min = min.min(mn);
        max = max.max(mx);
    }
    let world_aabb = Aabb::from_min_max(min.into(), max.into());

    let views = views
        .iter()
        .filter_map(|(entity, transform, camera)| {
            if camera.is_active {
                Some((entity, transform.affine()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for (transform, directional_light, mut cascades_config, mut cascades) in lights.iter_mut() {
        if !directional_light.shadows_enabled {
            continue;
        }

        // It is very important to the numerical and thus visual stability of shadows that
        // light_to_world has orthogonal upper-left 3x3 and zero translation.
        // Even though only the direction (i.e. rotation) of the light matters, we don't constrain
        // users to not change any other aspects of the transform - there's no guarantee
        // `transform.compute_matrix()` will give us a matrix with our desired properties.
        // Instead, we directly create a good matrix from just the rotation.
        let light_to_world = Affine3A::from_quat(transform.compute_transform().rotation);
        let light_to_world_inverse = light_to_world.inverse();

        let n_bounds = cascades_config.bounds.len();
        cascades.cascades.clear();
        for (view_entity, view_to_world) in views.iter().copied() {
            let (mn, mx) = aabb_to_transformed_min_max(&world_aabb, &view_to_world.inverse());

            let camera_to_light_view = light_to_world_inverse * view_to_world;
            let view_cascades = cascades_config
                .bounds
                .iter_mut()
                .enumerate()
                .map(|(idx, far_bound)| {
                    let near = (n_bounds - idx) as f32 / n_bounds as f32;
                    let far = (n_bounds - idx - 1) as f32 / n_bounds as f32;
                    let near = lerp(mn.z, mx.z, near);
                    let far = lerp(mn.z, mx.z, far);
                    *far_bound = -far;

                    calculate_cascade(
                        min_max_props_to_corners(&mn, &mx, near, far),
                        directional_light_shadow_map.size as f32,
                        light_to_world.into(),
                        camera_to_light_view.into(),
                    )
                })
                .collect();
            cascades.cascades.insert(view_entity, view_cascades);
        }
    }
}

fn min_max_props_to_corners(min: &Vec3A, max: &Vec3A, near: f32, far: f32) -> [Vec3A; 8] {
    [
        Vec3A::new(max.x, min.y, near),
        Vec3A::new(max.x, max.y, near),
        Vec3A::new(min.x, max.y, near),
        Vec3A::new(min.x, min.y, near),
        Vec3A::new(max.x, min.y, far),
        Vec3A::new(max.x, max.y, far),
        Vec3A::new(min.x, max.y, far),
        Vec3A::new(min.x, min.y, far),
    ]
}
