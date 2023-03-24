//! Controls how the atmosphere and sky look.

use bevy::prelude::*;

use crate::asset_management::palette::environment::SKY_SUNNY;

/// Logic and resources to modify the sky and atmosphere.
pub(super) struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(SKY_SUNNY));
    }
}
