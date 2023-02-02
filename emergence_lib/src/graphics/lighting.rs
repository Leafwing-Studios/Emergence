//! Lights and lighting.

use bevy::prelude::*;

/// Handles all lighting logic
pub(super) struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 5.,
            color: Color::WHITE,
        });
    }
}
