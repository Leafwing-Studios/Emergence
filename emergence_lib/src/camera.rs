use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PanCamPlugin::default())
            .add_startup_system_to_stage(StartupStage::Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera...");
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam::default());
}
