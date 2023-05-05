//! Controls how the atmosphere and sky look.

use bevy::prelude::*;

use crate::simulation::time::InGameTime;

/// Logic and resources to modify the sky and atmosphere.
pub(super) struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sky_color);
    }
}

/// Changes the `ClearColor` resource which drives the sky color based on the time of day.
fn animate_sky_color(mut clear_color: ResMut<ClearColor>, in_game_time: Res<InGameTime>) {
    clear_color.0 = in_game_time.time_of_day().sky_color();
}
