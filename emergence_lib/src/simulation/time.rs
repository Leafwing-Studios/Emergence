//! Controls in-game time: day-night cycles, weather and the passing of seasons

use core::fmt::Display;
use std::ops::{Div, Mul};

use bevy::prelude::*;
use derive_more::{Add, AddAssign, Display, Sub, SubAssign};
use leafwing_abilities::pool::MaxPoolLessThanZero;
use leafwing_abilities::prelude::Pool;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};

use crate::graphics::lighting::{Moon, Sun};
use crate::organisms::lifecycle::Lifecycle;
use crate::player_interaction::PlayerAction;

use super::{PauseState, SimulationSet};

/// Introduces temporal variation into the environment.
pub(crate) struct TemporalPlugin;

impl Plugin for TemporalPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PauseState>()
            .insert_resource(FixedTime::new_from_secs(1.0 / 30.))
            .add_systems(
                FixedUpdate,
                (
                    advance_in_game_time,
                    move_celestial_bodies,
                    record_elapsed_time_for_lifecycles,
                )
                    .chain()
                    .in_set(SimulationSet),
            )
            .add_systems(Update, pause_game)
            .init_resource::<InGameTime>();
    }
}

/// Stores the in game time.
#[derive(Resource)]
pub struct InGameTime {
    /// How much time has elapsed, in units of in-game days.
    elapsed_time: Days,
    /// The number of wall-clock seconds that should elapse per complete in-game day.
    seconds_per_day: f32,
}

/// A duration of time, in in-game days.
#[derive(
    Debug,
    Clone,
    Copy,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
)]
pub struct Days(pub f32);

impl Days {
    /// Represents a duration of 0 days.
    pub const ZERO: Self = Days(0.);
}

impl Div<f32> for Days {
    type Output = Days;

    fn div(self, rhs: f32) -> Self::Output {
        Days(self.0 / rhs)
    }
}

impl Mul<f32> for Days {
    type Output = Days;

    fn mul(self, rhs: f32) -> Self::Output {
        Days(self.0 * rhs)
    }
}

/// A discrete time of day.
///
/// These are evenly spaced throughout the 24 hour day.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// The sun is out
    Day,
    /// The sun is down
    Night,
}

impl TimeOfDay {
    /// Returns the time of day that is closest to the given fraction of a day.
    ///
    /// Values outside of [0.0, 1.0] are modulo'd to fit the range.
    pub fn from_fraction_of_day(fraction: f32) -> Self {
        if fraction < 0.7 {
            TimeOfDay::Day
        } else {
            TimeOfDay::Night
        }
    }
}

impl InGameTime {
    /// How many days have elapsed total?
    pub fn elapsed_days(&self) -> f32 {
        self.elapsed_time.0
    }
    /// How many days have elapsed total, rounded to the nearest day?
    pub fn rounded_elapsed_days(&self) -> u64 {
        self.elapsed_time.0.floor() as u64
    }

    /// How far are we through the day?
    ///
    /// This begins at dawn, and ends at dawn the next day.
    pub fn fraction_of_day(&self) -> f32 {
        self.elapsed_time.0 % 1.0
    }

    /// What time of day is it?
    pub fn time_of_day(&self) -> TimeOfDay {
        TimeOfDay::from_fraction_of_day(self.fraction_of_day())
    }

    /// What time is it, in 24 hour time?
    pub fn twenty_four_hour_time(&self) -> f32 {
        // Correct for different time systems: fraction of day begins at dawn,
        // but 24 hour time begins at midnight.
        ((self.fraction_of_day() + 0.25) * 24.) % 24.
    }

    /// Returns the configured number of seconds per day.
    pub fn seconds_per_day(&self) -> f32 {
        self.seconds_per_day
    }
}

impl Display for InGameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} days elapsed\n{:.2}h ({})",
            self.rounded_elapsed_days(),
            self.twenty_four_hour_time(),
            self.time_of_day()
        )
    }
}

impl Default for InGameTime {
    fn default() -> Self {
        InGameTime {
            elapsed_time: Days(0.0),
            seconds_per_day: 300.,
        }
    }
}

/// Advances the in game time based on elapsed clock time when the game is not paused.
pub fn advance_in_game_time(time: Res<FixedTime>, mut in_game_time: ResMut<InGameTime>) {
    let delta = Days(time.period.as_secs_f32() / in_game_time.seconds_per_day);
    in_game_time.elapsed_time += delta;
}

/// Moves the sun and moon based on the in-game time
fn move_celestial_bodies(
    mut sun_query: Query<&mut Visibility, (With<Sun>, Without<Moon>)>,
    mut moon_query: Query<&mut Visibility, With<Moon>>,
    in_game_time: Res<InGameTime>,
) {
    let mut sun_visibility = sun_query.single_mut();
    let mut moon_visibility = moon_query.single_mut();

    match in_game_time.time_of_day() {
        TimeOfDay::Day => {
            *sun_visibility = Visibility::Visible;
            *moon_visibility = Visibility::Hidden;
        }
        TimeOfDay::Night => {
            *sun_visibility = Visibility::Hidden;
            *moon_visibility = Visibility::Visible;
        }
    }
}

/// Pauses and unpauses the game when prompted by player input
fn pause_game(
    current_pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    player_actions: Res<ActionState<PlayerAction>>,
) {
    if player_actions.just_pressed(PlayerAction::TogglePause) {
        next_pause_state.set(match current_pause_state.get() {
            PauseState::Paused => PauseState::Playing,
            PauseState::Playing => PauseState::Paused,
        });
    }
}

/// A [`Pool`] of [`Days`], which builds up and will eventually be filled (at which point some event will occur).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TimePool {
    /// The current quantity of this pool.
    current: Days,
    /// The maximum quantity of this pool.
    max: Days,
}

impl TimePool {
    /// Creates a new [`TimePool`] with the given maximum capacity.
    pub fn simple(max: f32) -> Self {
        Self {
            current: Days(0.),
            max: Days(max),
        }
    }

    /// Is this pool full?
    pub(crate) fn is_full(&self) -> bool {
        self.current >= self.max
    }
}

impl Pool for TimePool {
    type Quantity = Days;

    const ZERO: Days = Days(0.);

    fn new(
        current: Self::Quantity,
        max: Self::Quantity,
        _regen_per_second: Self::Quantity,
    ) -> Self {
        Self { current, max }
    }

    fn current(&self) -> Self::Quantity {
        self.current
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
        let actual = Days(new_quantity.0.clamp(0., self.max.0));
        self.current = actual;
        actual
    }

    fn max(&self) -> Self::Quantity {
        self.max
    }

    fn set_max(
        &mut self,
        new_max: Self::Quantity,
    ) -> Result<(), leafwing_abilities::pool::MaxPoolLessThanZero> {
        if new_max < Self::ZERO {
            Err(MaxPoolLessThanZero)
        } else {
            self.max = new_max;
            self.set_current(self.current);
            Ok(())
        }
    }

    fn regen_per_second(&self) -> Self::Quantity {
        panic!("Time does not regenerate")
    }

    fn set_regen_per_second(&mut self, _new_regen_per_second: Self::Quantity) {
        panic!("Time does not regenerate")
    }
}

/// Advances life cycles accorded to elapsed in-game time
fn record_elapsed_time_for_lifecycles(
    mut query: Query<&mut Lifecycle>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
) {
    for mut lifecycle in query.iter_mut() {
        let delta_days = Days(fixed_time.period.as_secs_f32() / in_game_time.seconds_per_day);
        lifecycle.record_elapsed_time(delta_days);
    }
}
