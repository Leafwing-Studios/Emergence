//! Varies the weather each day.

use bevy::prelude::*;
use derive_more::Display;
use emergence_macros::IterableEnum;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate as emergence_lib;
use crate::simulation::time::InGameTime;

/// A plugin that handles weather.
pub(crate) struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentWeather>().add_systems(
            (set_daily_weather,)
                .in_set(super::SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// The current weather.
#[derive(Resource)]
pub struct CurrentWeather {
    /// The day that the weather was last updated.
    last_updated: u32,
    /// The current weather.
    weather: Weather,
}

impl Default for CurrentWeather {
    fn default() -> Self {
        Self {
            last_updated: 0,
            weather: Weather::Clear,
        }
    }
}

impl CurrentWeather {
    /// Initializes this resource with the provided `weather`.
    #[cfg(test)]
    pub(crate) fn new(weather: Weather) -> Self {
        Self {
            last_updated: 0,
            weather,
        }
    }

    /// Access the current weather.
    pub(crate) fn get(&self) -> Weather {
        self.weather
    }
}

/// A type of weather.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, IterableEnum)]
pub(crate) enum Weather {
    /// A clear day.
    Clear,
    /// A cloudy day.
    Cloudy,
    /// A rainy day.
    Rainy,
}

impl Weather {
    /// Chooses a random weather.
    fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen_range(0..3) {
            0 => Self::Clear,
            1 => Self::Cloudy,
            2 => Self::Rainy,
            _ => unreachable!(),
        }
    }

    /// The relative rate of precipitation for this kind of weather.
    ///
    /// The precipitation rate of [`Weather::Clear`] is defined to be 0.0.
    /// The precipitation rate of [`Weather::Rainy`] is defined to be 1.0.
    pub(crate) fn precipitation_rate(self) -> f32 {
        match self {
            Self::Clear => 0.,
            Self::Cloudy => 0.0,
            Self::Rainy => 1.,
        }
    }
}

/// Sets the weather for the day.
fn set_daily_weather(in_game_time: Res<InGameTime>, mut current_weather: ResMut<CurrentWeather>) {
    let current_day = in_game_time.elapsed_days() as u32;
    if current_weather.last_updated != current_day {
        current_weather.last_updated = current_day;
        let rng = &mut rand::thread_rng();
        current_weather.weather = Weather::random(rng);
    }
}
