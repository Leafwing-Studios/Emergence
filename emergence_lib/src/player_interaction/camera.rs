//! Camera controls and movement.
//!
//! This RTS-style camera can zoom, pan and rotate.

use bevy::prelude::*;
use leafwing_input_manager::axislike::SingleAxis;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::VirtualDPad;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputManagerBundle;

use super::InteractionSystem;

/// Camera logic
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<CameraAction>::default())
            .add_startup_system_to_stage(StartupStage::Startup, setup)
            .add_system(camera_movement.label(InteractionSystem::MoveCamera));
    }
}

/// Spawns a [`Camera3dBundle`] and sets up the [`InputManagerBundle`]s that handle camera motion
fn setup(mut commands: Commands) {
    // FIXME: swap to z-up coordinates. Blocked on https://github.com/ManevilleF/hexx/issues/10
    let initial_transform = Transform::from_xyz(0.0, 1.0, 0.0);

    commands
        .spawn(Camera3dBundle {
            transform: initial_transform,
            ..Default::default()
        })
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
        .insert(CameraSettings::default());
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
    /// The rate at which the camera rotates.
    ///
    /// Should always be positive.
    rotation_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            zoom_speed: 500.,
            pan_speed: 50.,
            rotation_speed: 1.,
        }
    }
}

/// The scale that controls the amount the camera will move in the x direction
const ZOOM_PAN_SCALE: f32 = 0.5;

/// Handles camera motion
fn camera_movement(
    mut camera_query: Query<
        (&mut Transform, &ActionState<CameraAction>, &CameraSettings),
        With<Camera3d>,
    >,
    time: Res<Time>,
) {
    let (mut transform, camera_actions, settings) = camera_query.single_mut();

    // Zoom
    if camera_actions.pressed(CameraAction::Zoom) {
        let delta_zoom = camera_actions.value(CameraAction::Zoom)
            * time.delta_seconds()
            * settings.zoom_speed
            * ZOOM_PAN_SCALE;

        // Zoom in / out on whatever we're looking at
        let delta = -transform.up() * delta_zoom;

        transform.translation += delta;
    }

    // Pan
    if camera_actions.pressed(CameraAction::Pan) {
        let dual_axis_data = camera_actions.axis_pair(CameraAction::Pan).unwrap();
        let base_xy = dual_axis_data.xy();
        let scaled_xy = base_xy * time.delta_seconds() * settings.pan_speed;

        let x_motion = transform.right() * scaled_xy.x;
        let y_motion = transform.forward() * scaled_xy.y;

        transform.translation += x_motion + y_motion;
    }

    // Rotate
    if camera_actions.pressed(CameraAction::RotateLeft) {
        transform.rotate_local_y(settings.rotation_speed * time.delta_seconds());
    }

    if camera_actions.pressed(CameraAction::RotateRight) {
        transform.rotate_local_y(-settings.rotation_speed * time.delta_seconds());
    }
}
