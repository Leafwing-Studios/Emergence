//! Camera controls and movement.
//!
//! This RTS-style camera can zoom, pan and rotate.

use std::f32::consts::PI;
use std::f32::consts::TAU;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_mod_raycast::RaycastSource;
use leafwing_input_manager::prelude::ActionState;

use crate::simulation::geometry::Facing;
use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;
use crate::structures::ghost::Ghost;
use crate::structures::StructureId;
use crate::terrain::Terrain;
use crate::units::UnitId;

use self::speed::Speed;

use super::selection::CurrentSelection;
use super::InteractionSystem;
use super::PlayerAction;

/// Camera logic
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::Startup, setup_camera)
            .add_system(set_camera_inclination.before(translate_camera))
            .add_system(mousewheel_zoom.before(translate_camera))
            .add_system(rotate_camera.before(InteractionSystem::MoveCamera))
            .add_system(translate_camera.before(InteractionSystem::MoveCamera))
            .add_system(move_camera_to_goal.label(InteractionSystem::MoveCamera));
    }
}

/// The distance from the origin that the camera begins at.
///
/// Should be between the default values of [`CameraSettings`] `min_zoom` and `max_zoom`.
const STARTING_DISTANCE_FROM_ORIGIN: f32 = 30.;

/// Spawns a [`Camera3dBundle`] and associated camera components.
fn setup_camera(mut commands: Commands, map_geometry: Res<MapGeometry>) {
    let focus = CameraFocus::default();
    let settings = CameraSettings::default();
    let facing = Facing::default();
    let planar_angle = facing.direction.angle(&map_geometry.layout.orientation);

    let transform = compute_camera_transform(&focus, planar_angle, settings.inclination);

    commands
        .spawn(Camera3dBundle {
            transform,
            ..Default::default()
        })
        .insert(settings)
        .insert(focus)
        .insert(Facing::default())
        .insert(RaycastSource::<Terrain>::new())
        .insert(RaycastSource::<StructureId>::new())
        .insert(RaycastSource::<UnitId>::new())
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
    /// The distance from the focus to the camera.
    zoom: f32,
}

impl Default for CameraFocus {
    fn default() -> Self {
        CameraFocus {
            translation: Vec3::ZERO,
            zoom: STARTING_DISTANCE_FROM_ORIGIN,
        }
    }
}

/// Configure how the camera moves and feels.
#[derive(Component)]
struct CameraSettings {
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
    min_zoom: f32,
    /// The maximum distance that the camera can be from its focus.
    ///
    /// Should always be positive, and less than `max_zoom`.
    max_zoom: f32,
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
            zoom_speed: Speed::new(50., 100.0, 200.0),
            pan_speed: Speed::new(100., 100.0, 150.0),
            rotation_speed: Speed::new(0.3, 3.0, 5.0),
            min_zoom: 7.,
            max_zoom: 100.,
            float_radius: 3,
            inclination: 0.7 * PI / 2.,
            inclination_speed: 1.,
        }
    }
}

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

/// Pan and zoom the camera
fn translate_camera(
    mut camera_query: Query<(&mut CameraFocus, &Facing, &mut CameraSettings), With<Camera3d>>,
    time: Res<Time>,
    actions: Res<ActionState<PlayerAction>>,
    map_geometry: Res<MapGeometry>,
    selection: Res<CurrentSelection>,
    tile_pos_query: Query<&TilePos>,
) {
    let (mut focus, facing, mut settings) = camera_query.single_mut();

    // Zoom
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
    focus.zoom = (focus.zoom + delta_zoom).clamp(settings.min_zoom, settings.max_zoom);

    // Pan
    if actions.pressed(PlayerAction::Pan) {
        let dual_axis_data = actions.axis_pair(PlayerAction::Pan).unwrap();
        let base_xy = dual_axis_data.xy();
        let scaled_xy =
            base_xy * time.delta_seconds() * settings.pan_speed.delta(time.delta()) * focus.zoom;
        // Plane is XZ, but gamepads are XY
        let unoriented_translation = Vec3 {
            x: scaled_xy.y,
            y: 0.,
            z: scaled_xy.x,
        };

        let facing_angle = facing.direction.angle(&map_geometry.layout.orientation);
        let rotation = Quat::from_rotation_y(facing_angle);
        let oriented_translation = rotation.mul_vec3(unoriented_translation);

        focus.translation += oriented_translation;

        let nearest_tile_pos = TilePos::from_world_pos(focus.translation, &map_geometry);
        focus.translation.y = map_geometry.average_height(nearest_tile_pos, settings.float_radius);
    } else {
        settings.pan_speed.reset_speed();
    }

    // Snap to selected object
    if actions.pressed(PlayerAction::SnapToSelection) {
        let tile_to_snap_to = match &*selection {
            CurrentSelection::Ghost(entity)
            | CurrentSelection::Unit(entity)
            | CurrentSelection::Structure(entity) => Some(*tile_pos_query.get(*entity).unwrap()),
            CurrentSelection::Terrain(selected_tiles) => Some(selected_tiles.center()),
            CurrentSelection::None => None,
        };

        if let Some(target) = tile_to_snap_to {
            focus.translation = target.into_world_pos(&map_geometry);
        }
    }
}

/// Rotates the camera around the [`CameraFocus`].
fn rotate_camera(
    mut query: Query<&mut Facing, With<Camera3d>>,
    actions: Res<ActionState<PlayerAction>>,
) {
    let mut facing = query.single_mut();

    // Set facing
    if actions.just_pressed(PlayerAction::RotateCameraLeft) {
        facing.rotate_left();
    }

    if actions.just_pressed(PlayerAction::RotateCameraRight) {
        facing.rotate_right();
    }
}

// Move the camera around a central point, constantly looking at it and maintaining a fixed distance.
fn move_camera_to_goal(
    mut query: Query<(&mut Transform, &Facing, &CameraFocus, &mut CameraSettings), With<Camera3d>>,
    map_geometry: Res<MapGeometry>,
    mut intermediate_planar_angle: Local<f32>,
    time: Res<Time>,
) {
    /// Differences in target angle below this amount are ignored.
    ///
    /// This reduces camera jitter.
    const ROTATION_EPSILON: f32 = 1e-3;

    let (mut transform, facing, focus, mut settings) = query.single_mut();

    // Determine our goal
    let final_planar_angle = facing.direction.angle(&map_geometry.layout.orientation);

    // Compute the shortest distance between them
    // Formula from https://stackoverflow.com/a/7869457
    let signed_rotation =
        (final_planar_angle - *intermediate_planar_angle + PI).rem_euclid(TAU) - PI;

    if signed_rotation.abs() < ROTATION_EPSILON {
        *intermediate_planar_angle = final_planar_angle;
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
        *intermediate_planar_angle += actual_signed_distance;
    }

    // Replace the previous transform
    *transform = compute_camera_transform(focus, *intermediate_planar_angle, settings.inclination);
}

/// Computes the camera transform such that it is looking at `focus`
fn compute_camera_transform(focus: &CameraFocus, planar_angle: f32, inclination: f32) -> Transform {
    // Always begin due "south" of the focus.
    let mut transform = Transform::from_translation(focus.translation + Vec3::NEG_X * focus.zoom);

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
