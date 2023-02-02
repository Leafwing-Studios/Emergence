//! Camera controls and movement.
//!
//! This RTS-style camera can zoom, pan and rotate.

use std::f32::consts::PI;

use bevy::prelude::*;
use leafwing_input_manager::axislike::SingleAxis;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::VirtualDPad;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputManagerBundle;

use crate::simulation::geometry::angle;
use crate::simulation::geometry::clockwise;
use crate::simulation::geometry::counterclockwise;
use crate::simulation::geometry::Facing;

use super::InteractionSystem;

/// Camera logic
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<CameraAction>::default())
            .add_startup_system_to_stage(StartupStage::Startup, setup)
            .add_system(
                rotate_camera
                    .label(InteractionSystem::MoveCamera)
                    // We rely on the updated focus information from this system
                    .after(translate_camera),
            )
            .add_system(translate_camera.label(InteractionSystem::MoveCamera));
    }
}

/// The distance from the origin that the camera begins at.
const STARTING_DISTANCE_FROM_ORIGIN: f32 = 15.;

/// The angle in radians that the camera forms with the ground.
///
/// This value should be between 0 (horizontal) and PI / 2 (vertical).
const CAMERA_ANGLE: f32 = PI / 4.;

/// Spawns a [`Camera3dBundle`] and sets up the [`InputManagerBundle`]s that handle camera motion
fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(InputManagerBundle::<CameraAction> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), CameraAction::Pan)
                .insert(VirtualDPad::arrow_keys(), CameraAction::Pan)
                .insert(SingleAxis::mouse_wheel_y(), CameraAction::Zoom)
                .insert(KeyCode::Q, CameraAction::RotateLeft)
                .insert(KeyCode::E, CameraAction::RotateRight)
                .build(),
            ..default()
        })
        .insert(CameraSettings::default())
        .insert(CameraFocus::default())
        .insert(Facing::default());
}

/// Actions that manipulate the camera
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraAction {
    /// Move the camera from side to side
    Pan,
    /// Reveal more or less of the map by pulling the camera away or moving it closer
    Zoom,
    /// Rotates the camera counterclockwise
    RotateLeft,
    /// Rotates the camera clockwise
    RotateRight,
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
    /// Scaling factor for how fast the camera zooms in and out.
    ///
    /// Should always be positive.
    zoom_speed: f32,
    /// Scaling factor for how fast the camera moves from side to side.
    ///
    /// Should always be positive.
    pan_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            zoom_speed: 500.,
            pan_speed: 50.,
        }
    }
}

/// The scale that controls the amount the camera will move in the x direction
const ZOOM_PAN_SCALE: f32 = 0.5;

/// Pan and zoom the camera
fn translate_camera(
    mut camera_query: Query<
        (
            &mut CameraFocus,
            &Transform,
            &ActionState<CameraAction>,
            &CameraSettings,
        ),
        With<Camera3d>,
    >,
    time: Res<Time>,
) {
    let (mut focus, transform, camera_actions, settings) = camera_query.single_mut();

    // Zoom
    if camera_actions.pressed(CameraAction::Zoom) {
        let delta_zoom = camera_actions.value(CameraAction::Zoom)
            * time.delta_seconds()
            * settings.zoom_speed
            * ZOOM_PAN_SCALE;

        // Zoom in / out on whatever we're looking at
        focus.zoom -= delta_zoom;
    }

    // Pan
    if camera_actions.pressed(CameraAction::Pan) {
        let dual_axis_data = camera_actions.axis_pair(CameraAction::Pan).unwrap();
        let base_xy = dual_axis_data.xy();
        let scaled_xy = base_xy * time.delta_seconds() * settings.pan_speed;

        let x_motion = transform.right() * scaled_xy.x;
        let y_motion = transform.forward() * scaled_xy.y;

        focus.translation += x_motion + y_motion;
    }
}

/// Rotates the camera around the [`CameraFocus`].
fn rotate_camera(
    mut query: Query<
        (
            &mut Transform,
            &mut Facing,
            &CameraFocus,
            &ActionState<CameraAction>,
        ),
        With<Camera3d>,
    >,
) {
    let (mut transform, mut facing, focus, camera_actions) = query.single_mut();

    // Set facing
    if camera_actions.just_pressed(CameraAction::RotateLeft) {
        facing.direction = counterclockwise(facing.direction);
    }

    if camera_actions.just_pressed(CameraAction::RotateRight) {
        facing.direction = clockwise(facing.direction);
    }

    // Goal: move the camera around a central point

    // Always begin due "south" of the focus.
    let mut new_transform =
        Transform::from_translation(focus.translation + Vec3::NEG_X * focus.zoom);

    // Tilt up
    new_transform.translate_around(
        focus.translation,
        Quat::from_axis_angle(Vec3::NEG_Z, CAMERA_ANGLE),
    );

    // Rotate around on the xz plane
    let planar_angle = angle(facing.direction);
    new_transform.translate_around(focus.translation, Quat::from_rotation_y(planar_angle));

    // Look at that central point
    new_transform.look_at(focus.translation, Vec3::Y);

    // Replace the previous transform
    *transform = new_transform;
}
