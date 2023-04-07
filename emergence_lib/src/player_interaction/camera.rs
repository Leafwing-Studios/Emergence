//! Camera controls and movement.
//!
//! This RTS-style camera can zoom, pan and rotate.

use std::f32::consts::PI;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_mod_raycast::RaycastSource;
use leafwing_input_manager::orientation::Rotation;
use leafwing_input_manager::prelude::ActionState;

use crate::construction::ghosts::Ghost;
use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;
use crate::structures::structure_manifest::Structure;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_manifest::Unit;

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
            .add_system(
                set_camera_focus
                    // Allow users to break out of CameraMode::Follow by moving the camera manually
                    .before(rotate_camera)
                    .before(pan_camera)
                    // Avoid jittering when the camera is following a unit
                    .after(drag_camera),
            )
            .add_system(set_camera_inclination.before(InteractionSystem::MoveCamera))
            .add_system(rotate_camera.before(InteractionSystem::MoveCamera))
            .add_system(pan_camera.before(InteractionSystem::MoveCamera))
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

    let transform = compute_camera_transform(&focus, settings.facing, settings.inclination);
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
        .insert(RaycastSource::<Terrain>::new())
        .insert(RaycastSource::<Structure>::new())
        .insert(RaycastSource::<Unit>::new())
        .insert(RaycastSource::<(Ghost, Structure)>::new());
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

/// Configure how the camera moves and feels.
#[derive(Component)]
pub(crate) struct CameraSettings {
    /// How should this camera behave?
    pub(crate) camera_mode: CameraMode,
    /// Controls how fast the camera zooms in and out.
    zoom_speed: Speed,
    /// Controls the rate that the camera can moves from side to side.
    pan_speed: Speed,
    /// The angle in radians that the camera forms around the y axis.
    facing: Rotation,
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
    inclination: Rotation,
    /// The rate in radians per second that the inclination changes.
    inclination_speed: Speed,
    /// How much should dragging the mouse rotate the camera?
    drag_ratio: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            camera_mode: CameraMode::Free,
            zoom_speed: Speed::new(400., 300.0, 1000.0),
            pan_speed: Speed::new(10., 20.0, 20.0),
            rotation_speed: Speed::new(1.0, 2.0, 4.0),
            min_zoom: 10.,
            max_zoom: 500.,
            float_radius: 3,
            facing: Rotation::default(),
            inclination: Rotation::from_radians(0.5 * PI / 2.),
            inclination_speed: Speed::new(0.5, 1.0, 2.0),
            drag_ratio: 0.05,
        }
    }
}

/// Controls how the camera moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CameraMode {
    /// The camera is free to move around the map.
    Free,
    /// The camera is following the selected unit.
    FollowUnit,
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
    actions: Res<ActionState<PlayerAction>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut CameraSettings>,
    time: Res<Time>,
) {
    if actions.pressed(PlayerAction::DragCamera) {
        let mut settings = camera_query.single_mut();
        let rotation_rate = settings.rotation_speed.delta(time.delta()) * settings.drag_ratio;
        let inclination_rate = settings.inclination_speed.delta(time.delta()) * settings.drag_ratio;

        for mouse_motion in mouse_motion_events.iter() {
            settings.facing += Rotation::from_radians(mouse_motion.delta.x * rotation_rate);
            let proposed_inclination =
                settings.inclination.into_radians() + mouse_motion.delta.y * inclination_rate;
            let actual_inclination = proposed_inclination.clamp(0.0, PI / 2. - 1e-6);
            settings.inclination = Rotation::from_radians(actual_inclination);
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
        settings.inclination_speed.delta(time.delta())
    } else if actions.pressed(PlayerAction::TiltCameraDown) {
        -settings.inclination_speed.delta(time.delta())
    } else {
        return;
    };
    let proposed = settings.inclination.into_radians() + delta;
    // Cannot actually use PI/2., as this results in a singular matrix which causes look_at to fail in weird ways
    let actual = proposed.clamp(0.0, PI / 2. - 1e-6);
    settings.inclination = Rotation::from_radians(actual);
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

/// Sets the tile that the camera is  camera's focus.
fn set_camera_focus(
    actions: Res<ActionState<PlayerAction>>,
    selection: Res<CurrentSelection>,
    tile_pos_query: Query<&TilePos>,
    map_geometry: Res<MapGeometry>,
    unit_query: Query<&Transform>,
    mut camera_query: Query<(&mut CameraFocus, &mut CameraSettings), With<Camera3d>>,
) {
    let (mut focus, mut settings) = camera_query.single_mut();

    // Snap to selected object
    if actions.pressed(PlayerAction::CenterCameraOnSelection)
        || settings.camera_mode == CameraMode::FollowUnit
    {
        let tile_to_snap_to = match &*selection {
            CurrentSelection::GhostStructure(entity)
            | CurrentSelection::Unit(entity)
            | CurrentSelection::Structure(entity) => Some(*tile_pos_query.get(*entity).unwrap()),
            CurrentSelection::Terrain(selected_tiles) => Some(selected_tiles.center()),
            CurrentSelection::None => None,
        };

        if let Some(target) = tile_to_snap_to {
            focus.translation = target.top_of_tile(&map_geometry);
        }
    }

    // Also rotate the camera to match the orientation of the unit we're following
    if settings.camera_mode == CameraMode::FollowUnit {
        if let CurrentSelection::Unit(entity) = &*selection {
            let unit_transform = unit_query.get(*entity).unwrap();
            let quat = unit_transform.rotation;
            let euler = quat.to_euler(EulerRot::YXZ);
            let angle_around_y = euler.0;
            settings.facing = Rotation::from_radians(angle_around_y);
        } else {
            // If we don't have a unit selected, go back to free camera mode
            settings.camera_mode = CameraMode::Free;
        }
    }
}

/// Pan the camera
fn pan_camera(
    mut camera_query: Query<(&Transform, &mut CameraFocus, &mut CameraSettings), With<Camera3d>>,
    time: Res<Time>,
    actions: Res<ActionState<PlayerAction>>,
    map_geometry: Res<MapGeometry>,
) {
    let (transform, mut focus, mut settings) = camera_query.single_mut();

    // Pan
    if actions.pressed(PlayerAction::Pan) {
        settings.camera_mode = CameraMode::Free;

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

        let facing_angle = settings.facing.into_radians();
        let rotation = Quat::from_rotation_y(facing_angle);
        let oriented_translation = rotation.mul_vec3(unoriented_translation);

        focus.translation += oriented_translation;

        let nearest_tile_pos = TilePos::from_world_pos(transform.translation, &map_geometry);
        focus.translation.y = map_geometry.average_height(nearest_tile_pos, settings.float_radius);
    } else {
        settings.pan_speed.reset_speed();
    }
}

/// Rotates the camera around the [`CameraFocus`].
fn rotate_camera(
    mut query: Query<&mut CameraSettings, With<Camera3d>>,
    actions: Res<ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    let mut settings = query.single_mut();

    let delta = settings.rotation_speed.delta(time.delta());

    // Set facing
    if actions.pressed(PlayerAction::RotateCameraLeft) {
        settings.camera_mode = CameraMode::Free;
        settings.facing -= Rotation::from_radians(delta);
    }

    if actions.pressed(PlayerAction::RotateCameraRight) {
        settings.camera_mode = CameraMode::Free;
        settings.facing += Rotation::from_radians(delta);
    }
}

/// Move the camera around a central point, constantly looking at it and maintaining a fixed distance.
fn move_camera_to_goal(
    mut query: Query<(&mut Transform, &CameraFocus, &CameraSettings), With<Camera3d>>,
) {
    let (mut transform, focus, settings) = query.single_mut();

    // Replace the previous transform
    *transform = compute_camera_transform(focus, settings.facing, settings.inclination);
}

/// Computes the camera transform such that it is looking at `focus`
fn compute_camera_transform(
    focus: &CameraFocus,
    facing: Rotation,
    inclination: Rotation,
) -> Transform {
    // Always begin due "south" of the focus.
    let mut transform =
        Transform::from_translation(focus.translation + Vec3::NEG_X * focus.distance);

    // Tilt up
    transform.translate_around(
        focus.translation,
        Quat::from_axis_angle(Vec3::NEG_Z, inclination.into_radians()),
    );

    // Rotate left and right
    transform.translate_around(
        focus.translation,
        Quat::from_rotation_y(facing.into_radians()),
    );

    // Look at the focus
    transform.look_at(focus.translation, Vec3::Y);

    transform
}
