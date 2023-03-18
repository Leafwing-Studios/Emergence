//! Controls in-game time: day-night cycles, weather and the passing of seasons

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::graphics::lighting::CelestialBody;

/// Introduces temporal variation into the environment.
pub(super) struct TemporalPlugin;

impl Plugin for TemporalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((advance_in_game_time, move_sun).chain())
            .init_resource::<InGameTime>();
    }
}

/// Stores the in game time.
#[derive(Resource)]
struct InGameTime {
    /// How far are we through the day.
    ///
    /// - 0.0 is dawn
    /// - 0.25 is noon
    /// - 0.5 is dusk
    /// - 0.75 is midnight
    /// - 0.999 is just before dawn
    fraction_of_day: f32,
    /// The number of wall-clock seconds that should elapse per complete in-game day.
    seconds_per_day: f32,
}

impl Default for InGameTime {
    fn default() -> Self {
        InGameTime {
            fraction_of_day: 0.10,
            seconds_per_day: 60.,
        }
    }
}

/// Advances the in game time based on elapsed clock time when the game is not paused.
fn advance_in_game_time(time: Res<Time>, mut in_game_time: ResMut<InGameTime>) {
    in_game_time.fraction_of_day += time.delta_seconds() / in_game_time.seconds_per_day;
    in_game_time.fraction_of_day = in_game_time.fraction_of_day % 1.0;
}

/// Moves the sun based on the in-game time
fn move_sun(mut query: Query<&mut CelestialBody>, in_game_time: Res<InGameTime>) {
    let mut celestial_body = query.single_mut();
    // Scale the progress by TAU to get a full rotation.
    // Offset by PI / 2 to compensate for the fact that 0 represents the noon sun
    celestial_body.progress = in_game_time.fraction_of_day * TAU - PI / 2.;
}
