//! Camera controls and movement.
//!
//! This RTS-style camera can zoom and pan.

use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

/// Camera logic
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PanCamPlugin::default())
            .add_startup_system_to_stage(StartupStage::Startup, spawn_camera);
    }
}

/// Initialize the camera
fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera...");
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam::default());
}
