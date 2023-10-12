//! Computes the amount of light available at a given time.

use bevy::prelude::*;
use core::fmt::Display;

use emergence_macros::IterableEnum;
use serde::{Deserialize, Serialize};

use crate as emergence_lib;

use crate::simulation::{
    time::{InGameTime, TimeOfDay},
    weather::{CurrentWeather, Weather},
    SimulationSet,
};

use self::shade::{compute_received_light, compute_shade};

pub(crate) mod shade;

/// Systems and resources for computing light (in in-game quantities).
pub(super) struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TotalLight>().add_systems(
            Update,
            FixedUpdate,
            (compute_light, compute_shade, compute_received_light)
                .chain()
                .in_set(SimulationSet),
        );
    }
}

/// The total current amount of light available.
#[derive(Resource, Default, Debug)]
pub(crate) struct TotalLight(Illuminance);

impl Display for TotalLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A qualitative measurement of light intensity.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Serialize,
    Deserialize,
    IterableEnum,
)]
pub enum Illuminance {
    /// The tile is in complete darkness.
    Dark,
    /// The tile only has some light.
    DimlyLit,
    /// The tile has the full light of the sun.
    #[default]
    BrightlyLit,
}

impl Display for Illuminance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Illuminance::Dark => write!(f, "Dark"),
            Illuminance::DimlyLit => write!(f, "Dimly Lit"),
            Illuminance::BrightlyLit => write!(f, "Brightly Lit"),
        }
    }
}

/// Computes the amount of light available based on the weather and time of day.
fn compute_light(
    in_game_time: Res<InGameTime>,
    current_weather: Res<CurrentWeather>,
    mut total_light: ResMut<TotalLight>,
) {
    let time_of_day = in_game_time.time_of_day();

    total_light.0 = if time_of_day == TimeOfDay::Night {
        Illuminance::Dark
    } else {
        match current_weather.get() {
            Weather::Clear => Illuminance::BrightlyLit,
            Weather::Cloudy => Illuminance::DimlyLit,
            Weather::Rainy => Illuminance::DimlyLit,
        }
    }
}
