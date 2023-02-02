//! Camera controls and movement.
//!
//! This RTS-style camera can zoom and pan.

// use std::f32::consts::FRAC_PI_3;

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

/// Sets the angle the camera is oriented around the z-axis.
const CAM_ANG: f32 = 0.733;

/// Spawns a [`Camera3dBundle`] and sets up the [`InputManagerBundle`]s that handle camera motion
fn setup(mut commands: Commands) {
    // FIXME: swap to z-up coordinates. Blocked on https://github.com/ManevilleF/hexx/issues/10
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(20.0, 6.0, 30.0)
                .looking_at(Vec3::ZERO, -Vec3::Y)
                .with_rotation(Quat::from_euler(EulerRot::XZY, 0., 0., CAM_ANG)),
            ..Default::default()
        })
        .insert(InputManagerBundle::<CameraAction> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), CameraAction::Pan)
                .insert(VirtualDPad::arrow_keys(), CameraAction::Pan)
                .insert(SingleAxis::mouse_wheel_y(), CameraAction::Zoom)
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

/// Handles camera motion
fn camera_movement(
    mut camera_query: Query<
        (&mut Transform, &ActionState<CameraAction>, &CameraSettings),
        With<Camera3d>,
    >,
    time: Res<Time>,
) {
    let (mut camera_transform, camera_actions, settings) = camera_query.single_mut();

    // Zoom
    if camera_actions.pressed(CameraAction::Zoom) {
        // FIXME: swap to z-up
        let camera_actions = camera_actions.value(CameraAction::Zoom);
        let delta_x = camera_actions * time.delta_seconds() * settings.zoom_speed * ZOOM_PAN_SCALE;
        let delta_y = camera_actions * time.delta_seconds() * settings.pan_speed;
        // oriented from the POV that you're the player trying to zoom in to the game map
        camera_transform.translation.y -= delta_y;

        camera_transform.translation.x -= delta_x;
        camera_transform.translation.z -= delta_x;
    }

    // Pan
    if camera_actions.pressed(CameraAction::Pan) {
        let dual_axis_data = camera_actions.axis_pair(CameraAction::Pan).unwrap();
        let delta_x = dual_axis_data.x() * time.delta_seconds() * settings.pan_speed;
        let delta_y = dual_axis_data.y() * time.delta_seconds() * settings.pan_speed;

        camera_transform.translation.x += delta_x;
        // FIXME: swap to z-up
        camera_transform.translation.z -= delta_y;
    }
}
