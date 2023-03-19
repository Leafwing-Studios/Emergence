//! Controls in-game time: day-night cycles, weather and the passing of seasons

use core::f32::consts::{PI, TAU};
use core::fmt::Display;

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::graphics::lighting::CelestialBody;
use crate::player_interaction::PlayerAction;

use super::{PauseState, SimulationSet};

/// Introduces temporal variation into the environment.
pub(super) struct TemporalPlugin;

impl Plugin for TemporalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (advance_in_game_time, move_celestial_bodies)
                .chain()
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .add_system(pause_game)
        .init_resource::<InGameTime>();
    }
}

/// Stores the in game time.
#[derive(Resource)]
pub struct InGameTime {
    /// How much time has elapsed, in units of in-game days.
    elapsed_time: f32,
    /// The number of wall-clock seconds that should elapse per complete in-game day.
    seconds_per_day: f32,
}

impl InGameTime {
    /// How many days have elapsed total?
    pub fn elapsed_days(&self) -> u64 {
        self.elapsed_time.floor() as u64
    }

    /// How far are we through the day?
    ///
    /// - 0.0 is dawn
    /// - 0.25 is noon
    /// - 0.5 is dusk
    /// - 0.75 is midnight
    /// - 0.999 is just before dawn
    pub fn fraction_of_day(&self) -> f32 {
        self.elapsed_time % 1.0
    }

    /// What time is it, in 24 hour time?
    pub fn twenty_four_hour_time(&self) -> f32 {
        // Correct for different time systems: fraction of day begins at dawn,
        // but 24 hour time begins at midnight.
        ((self.fraction_of_day() + 0.25) * 24.) % 24.
    }
}

impl Display for InGameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} days elapsed\n{:.2}h",
            self.elapsed_days(),
            self.twenty_four_hour_time()
        )
    }
}

impl Default for InGameTime {
    fn default() -> Self {
        InGameTime {
            elapsed_time: 0.0,
            seconds_per_day: 60.,
        }
    }
}

/// Advances the in game time based on elapsed clock time when the game is not paused.
fn advance_in_game_time(time: Res<FixedTime>, mut in_game_time: ResMut<InGameTime>) {
    in_game_time.elapsed_time += time.period.as_secs_f32() / in_game_time.seconds_per_day;
}

/// Moves the sun and moon based on the in-game time
fn move_celestial_bodies(mut query: Query<&mut CelestialBody>, in_game_time: Res<InGameTime>) {
    for mut celestial_body in query.iter_mut() {
        // Take the modulo with respect to the period to get the revolution period correct
        let cycle_normalized_time = (in_game_time.elapsed_time % celestial_body.days_per_cycle)
            / celestial_body.days_per_cycle;

        // Scale the progress by TAU to get a full rotation.
        // Offset by PI / 2 to compensate for the fact that 0 represents the noon sun
        celestial_body.progress = cycle_normalized_time * TAU - PI / 2.;
    }
}

/// Pauses and unpauses the game when prompted by player input
fn pause_game(
    current_pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    player_actions: Res<ActionState<PlayerAction>>,
) {
    if player_actions.just_pressed(PlayerAction::TogglePause) {
        next_pause_state.set(match current_pause_state.0 {
            PauseState::Paused => PauseState::Playing,
            PauseState::Playing => PauseState::Paused,
        });
    }
}
