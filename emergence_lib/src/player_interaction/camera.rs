//! Camera controls and movement.
//!
//! This RTS-style camera can zoom and pan.

use bevy::prelude::*;
use leafwing_input_manager::axislike::SingleAxis;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
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
            .add_system(camera_movement.label(InteractionSystem::MoveCamera))
            .add_system(camera_zoom.label(InteractionSystem::MoveCamera));
    }
}

/// Spawns a [`Camera3dBundle`] and sets up the [`InputManagerBundle`]s that handle camera motion
fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, -Vec3::Z),
            ..Default::default()
        })
        .insert(InputManagerBundle::<CameraAction> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), CameraAction::Pan)
                .insert(VirtualDPad::arrow_keys(), CameraAction::Pan)
                .insert(SingleAxis::mouse_wheel_y(), CameraAction::Zoom)
                .build(),
            ..default()
        });
}

/// Actions that manipulate the camera
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraAction {
    /// Move the camera from side to side
    Pan,
    /// Reveal more or less of the map by pulling the camera away or moving it closer
    Zoom,
}

/// Handles camera zoom
fn camera_zoom() {}

/// Handles camera movement
fn camera_movement() {}
