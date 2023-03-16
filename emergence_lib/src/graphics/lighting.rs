//! Lights and lighting.

use bevy::prelude::*;

use crate::asset_management::palette::LIGHT_SUN;

/// Handles all lighting logic
pub(super) struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: LIGHT_SUN,
        })
        .add_startup_system(spawn_sun);
    }
}

/// Spawns a directional light source to illuminate the scene
fn spawn_sun(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: LIGHT_SUN,
            illuminance: 1.2e5,
            ..Default::default()
        },
        transform: Transform::from_xyz(30., 100., 30.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
