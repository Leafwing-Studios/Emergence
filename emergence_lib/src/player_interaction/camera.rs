//! Camera controls and movement.
//!
//! This RTS-style camera can zoom, pan and rotate.

use std::f32::consts::PI;
use std::f32::consts::TAU;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_mod_raycast::RaycastSource;
use leafwing_input_manager::orientation::Rotation;
use leafwing_input_manager::prelude::ActionState;

use crate::asset_management::manifest::Id;
use crate::asset_management::manifest::Structure;
use crate::asset_management::manifest::Terrain;
use crate::asset_management::manifest::Unit;
use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;
use crate::structures::construction::Ghost;

use self::speed::Speed;

use super::selection::CurrentSelection;
use super::InteractionSystem;
use super::PlayerAction;

/// Camera logic
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(mousewheel_zoom.before(zoom))
            .add_system(zoom)
            .add_system(
                drag_camera
                    .before(set_camera_inclination)
                    .before(rotate_camera),
            )
            .add_system(set_camera_inclination.before(InteractionSystem::MoveCamera))
            .add_system(rotate_camera.before(InteractionSystem::MoveCamera))
            .add_system(translate_camera.before(InteractionSystem::MoveCamera))
            .add_system(move_camera_to_goal.in_set(InteractionSystem::MoveCamera));
    }
}

/// The distance from the origin that the camera begins at.
///
/// Should be between the default values of [`CameraSettings`] `min_zoom` and `max_zoom`.
const STARTING_DISTANCE_FROM_ORIGIN: f32 = 30.;

/// Spawns a [`Camera3dBundle`] and associated camera components.
fn setup_camera(mut commands: Commands) {
    let focus = CameraFocus::default();
    let settings = CameraSettings::default();
    let facing = CameraFacing::default();
    let planar_angle = facing.planar_angle.into_radians();

    let transform = compute_camera_transform(&focus, planar_angle, settings.inclination);
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: 0.2,
        ..Default::default()
    });

    commands
        .spawn(Camera3dBundle {
            transform,
            projection,
            tonemapping: Tonemapping::TonyMcMapface,
            ..Default::default()
        })
        .insert(settings)
        .insert(focus)
        .insert(facing)
        .insert(RaycastSource::<Terrain>::new())
        .insert(RaycastSource::<Id<Structure>>::new())
        .insert(RaycastSource::<Id<Unit>>::new())
        .insert(RaycastSource::<Ghost>::new());
}

/// The position that the camera is looking at.
///
/// When panning and zooming, this struct is updated, rather than modifying the camera's [`Transform`] directly.
#[derive(Component, Debug)]
struct CameraFocus {
    /// The coordinate that the camera is looking at.
    ///
    /// This should be the top of the column at the center of the screen.
    translation: Vec3,
    /// The distance from the camera to the target
    distance: f32,
}

impl Default for CameraFocus {
    fn default() -> Self {
        CameraFocus {
            translation: Vec3::ZERO,
            distance: STARTING_DISTANCE_FROM_ORIGIN,
        }
    }
}

/// The direction that the camera is facing.
///
/// This is stored as a rotation around the vertical axis, in radians.
#[derive(Component, Debug, Default)]
struct CameraFacing {
    /// The angle in radians that the camera forms around the z-axis.
    planar_angle: Rotation,
}

/// Configure how the camera moves and feels.
#[derive(Component)]
pub(crate) struct CameraSettings {
    /// Controls how fast the camera zooms in and out.
    zoom_speed: Speed,
    /// Controls the rate that the camera can moves from side to side.
    pan_speed: Speed,
    /// Controls how fast the camera rotates around the vertical axis.
    ///
    /// Units are in radians per second.
    rotation_speed: Speed,
    /// The minimum distance that the camera can be from its focus.
    ///
    /// Should always be positive, and less than `max_zoom`.
    pub(crate) min_zoom: f32,
    /// The maximum distance that the camera can be from its focus.
    ///
    /// Should always be positive, and less than `max_zoom`.
    pub(crate) max_zoom: f32,
    /// How many tiles away from the focus should the camera take into consideration when computing the correct height?
    ///
    /// Increasing this value will result in a "smoother ride" over the hills and valleys of the map.
    float_radius: u32,
    /// The angle in radians that the camera forms with the ground.
    ///
    /// This value should be between 0 (horizontal) and PI / 2 (vertical).
    inclination: f32,
    /// The rate in radians per second that the inclination changes.
    ///
    /// This value should be positive.
    inclination_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            zoom_speed: Speed::new(400., 300.0, 1000.0),
            pan_speed: Speed::new(50., 100.0, 150.0),
            rotation_speed: Speed::new(1.0, 2.0, 4.0),
            min_zoom: 10.,
            max_zoom: 500.,
            float_radius: 3,
            inclination: 0.5 * PI / 2.,
            inclination_speed: 1.,
        }
    }
}

/// Contains the [`Speed`] struct.
///
/// Lives in a dedicated module to enforce privacy.
mod speed {
    use bevy::utils::Duration;

    /// Controls the rate of camera movement.
    ///
    /// Minimum speed is greater than
    pub(super) struct Speed {
        /// The minimum speed, in units per second
        min: f32,
        /// The current speed, in units per second
        current_speed: f32,
        /// The rate at which speed change, in units per second squared
        acceleration: f32,
        /// The maximum speed, in units per second
        max: f32,
    }

    impl Speed {
        /// Creates a new [`Speed`]
        ///
        /// # Panics
        ///
        /// Improper parameters will panic on construction.
        pub(super) fn new(min: f32, acceleration: f32, max: f32) -> Self {
            assert!(min > 0.);
            assert!(acceleration > 0.);
            assert!(max > 0.);

            assert!(min <= max);

            Speed {
                min,
                current_speed: min,
                acceleration,
                max,
            }
        }

        /// The amount that has changed in the elapsed `delta_time`.
        pub(super) fn delta(&mut self, delta_time: Duration) -> f32 {
            let delta_v = self.acceleration * delta_time.as_secs_f32();

            let proposed = self.current_speed + delta_v;
            self.current_speed = proposed.clamp(self.min, self.max);

            self.current_speed * delta_time.as_secs_f32()
        }

        /// Resets the current speed to the minimum value
        pub(super) fn reset_speed(&mut self) {
            self.current_speed = self.min;
        }
    }
}

/// Zoom the camera based on the mouse wheel
///
/// This is needed to normalize gamepad / keyboard and mouse wheel zoom rates.
fn mousewheel_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut actions: ResMut<ActionState<PlayerAction>>,
) {
    if let Some(first_event) = mouse_wheel_events.iter().next() {
        if first_event.y > 0. {
            actions.press(PlayerAction::ZoomIn);
        } else {
            actions.press(PlayerAction::ZoomOut);
        }
    }
    mouse_wheel_events.clear();
}

/// Use middle-mouse + drag to rotate the camera and tilt it up and down
fn drag_camera(
    mut actions: ResMut<ActionState<PlayerAction>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    /// Controls the deadzone for camera dragging
    const DRAG_THRESHOLD: f32 = 0.01;

    if actions.pressed(PlayerAction::DragCamera) {
        for event in mouse_motion_events.iter() {
            match event.delta.x {
                x if x > DRAG_THRESHOLD => actions.press(PlayerAction::RotateCameraRight),
                x if x < -DRAG_THRESHOLD => actions.press(PlayerAction::RotateCameraLeft),
                _ => (),
            }

            match event.delta.y {
                y if y > DRAG_THRESHOLD => actions.press(PlayerAction::TiltCameraUp),
                y if y < -DRAG_THRESHOLD => actions.press(PlayerAction::TiltCameraDown),
                _ => (),
            }
        }
    }
}

/// Sets the inclination of the camera
fn set_camera_inclination(
    mut camera_query: Query<&mut CameraSettings, With<Camera3d>>,
    actions: Res<ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    let mut settings = camera_query.single_mut();

    let delta = if actions.pressed(PlayerAction::TiltCameraUp) {
        settings.inclination_speed * time.delta_seconds()
    } else if actions.pressed(PlayerAction::TiltCameraDown) {
        -settings.inclination_speed * time.delta_seconds()
    } else {
        return;
    };
    // Cannot actually use PI/2., as this results in a singular matrix which causes look_at to fail in weird ways
    settings.inclination = (settings.inclination + delta).clamp(0.0, PI / 2. - 1e-6);
}

/// Zooms the camera in and out
fn zoom(
    mut camera_query: Query<(&mut CameraFocus, &mut CameraSettings), With<Camera3d>>,
    actions: Res<ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    let (mut focus, mut settings) = camera_query.single_mut();

    let delta_zoom = match (
        actions.pressed(PlayerAction::ZoomIn),
        actions.pressed(PlayerAction::ZoomOut),
    ) {
        (true, false) => -settings.zoom_speed.delta(time.delta()),
        (false, true) => settings.zoom_speed.delta(time.delta()),
        _ => {
            settings.zoom_speed.reset_speed();
            0.0
        }
    };

    // Zoom in / out on whatever we're looking at
    focus.distance = (focus.distance + delta_zoom).clamp(settings.min_zoom, settings.max_zoom);
}

/// Pan the camera
fn translate_camera(
    mut camera_query: Query<
        (
            &Transform,
            &mut CameraFocus,
            &CameraFacing,
            &mut CameraSettings,
        ),
        With<Camera3d>,
    >,
    time: Res<Time>,
    actions: Res<ActionState<PlayerAction>>,
    map_geometry: Res<MapGeometry>,
    selection: Res<CurrentSelection>,
    tile_pos_query: Query<&TilePos>,
) {
    let (transform, mut focus, facing, mut settings) = camera_query.single_mut();

    // Pan
    if actions.pressed(PlayerAction::Pan) {
        let dual_axis_data = actions.axis_pair(PlayerAction::Pan).unwrap();
        let base_xy = dual_axis_data.xy();
        let scaled_xy = base_xy
            * time.delta_seconds()
            * settings.pan_speed.delta(time.delta())
            * focus.distance;
        // Plane is XZ, but gamepads are XY
        let unoriented_translation = Vec3 {
            x: scaled_xy.y,
            y: 0.,
            z: scaled_xy.x,
        };

        let facing_angle = facing.planar_angle.into_radians();
        let rotation = Quat::from_rotation_y(facing_angle);
        let oriented_translation = rotation.mul_vec3(unoriented_translation);

        focus.translation += oriented_translation;

        let nearest_tile_pos = TilePos::from_world_pos(transform.translation, &map_geometry);
        focus.translation.y = map_geometry.average_height(nearest_tile_pos, settings.float_radius);
    } else {
        settings.pan_speed.reset_speed();
    }

    // Snap to selected object
    if actions.pressed(PlayerAction::CenterCameraOnSelection) {
        let tile_to_snap_to = match &*selection {
            CurrentSelection::Ghost(entity)
            | CurrentSelection::Unit(entity)
            | CurrentSelection::Structure(entity) => Some(*tile_pos_query.get(*entity).unwrap()),
            CurrentSelection::Terrain(selected_tiles) => Some(selected_tiles.center()),
            CurrentSelection::None => None,
        };

        if let Some(target) = tile_to_snap_to {
            focus.translation = target.top_of_tile(&map_geometry);
        }
    }
}

/// Rotates the camera around the [`CameraFocus`].
fn rotate_camera(
    mut query: Query<(&mut CameraFacing, &mut CameraSettings), With<Camera3d>>,
    actions: Res<ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    let (mut facing, mut settings) = query.single_mut();

    let delta = settings.rotation_speed.delta(time.delta());

    // Set facing
    if actions.pressed(PlayerAction::RotateCameraLeft) {
        facing.planar_angle -= Rotation::from_radians(delta);
    }

    if actions.pressed(PlayerAction::RotateCameraRight) {
        facing.planar_angle += Rotation::from_radians(delta);
    }
}

/// Move the camera around a central point, constantly looking at it and maintaining a fixed distance.
fn move_camera_to_goal(
    mut query: Query<
        (
            &mut Transform,
            &CameraFacing,
            &CameraFocus,
            &mut CameraSettings,
        ),
        With<Camera3d>,
    >,
    mut cached_planar_angle: Local<Option<f32>>,
    time: Res<Time>,
) {
    /// Differences in target angle below this amount are ignored.
    ///
    /// This reduces camera jitter.
    const ROTATION_EPSILON: f32 = 1e-3;

    let (mut transform, facing, focus, mut settings) = query.single_mut();

    // Determine our goal
    let final_planar_angle = facing.planar_angle.into_radians();

    // The cached planar angle must begin uninitialized: otherwise changes to the default facing will result in a delay as  we pan towards it.
    // If it was uninitialized, start it at the final location.
    let intermediate_planar_angle = cached_planar_angle.unwrap_or(final_planar_angle);

    // Compute the shortest distance between them
    // Formula from https://stackoverflow.com/a/7869457
    let signed_rotation =
        (final_planar_angle - intermediate_planar_angle + PI).rem_euclid(TAU) - PI;

    if signed_rotation.abs() < ROTATION_EPSILON {
        *cached_planar_angle = Some(final_planar_angle);
    } else {
        // Compute the correct rotation
        let max_rotation = settings.rotation_speed.delta(time.delta());

        // Make sure not to overshoot
        let actual_signed_distance = if signed_rotation > 0. {
            signed_rotation.min(max_rotation)
        } else {
            signed_rotation.max(-max_rotation)
        };

        // Actually mutate the intermediate angle
        *cached_planar_angle = Some(intermediate_planar_angle + actual_signed_distance);
    }

    // Replace the previous transform
    *transform = compute_camera_transform(focus, intermediate_planar_angle, settings.inclination);
}

/// Computes the camera transform such that it is looking at `focus`
fn compute_camera_transform(focus: &CameraFocus, planar_angle: f32, inclination: f32) -> Transform {
    // Always begin due "south" of the focus.
    let mut transform =
        Transform::from_translation(focus.translation + Vec3::NEG_X * focus.distance);

    // Tilt up
    transform.translate_around(
        focus.translation,
        Quat::from_axis_angle(Vec3::NEG_Z, inclination),
    );

    // Rotate left and right
    transform.translate_around(focus.translation, Quat::from_rotation_y(planar_angle));

    // Look at the focus
    transform.look_at(focus.translation, Vec3::Y);

    transform
}
