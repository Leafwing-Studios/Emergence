//! Water dynamics for the ocean.

use bevy::prelude::*;

use crate::simulation::{
    geometry::Height,
    time::{Days, InGameTime},
};

use super::WaterConfig;

/// Controls the dynamics of [`tides`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TideSettings {
    /// The amplitude of the tide.
    pub amplitude: Height,
    /// The period of the tide.
    pub period: Days,
    /// The minimum water level of the ocean
    pub minimum: Height,
}

/// Stores data about the current state of the ocean.
#[derive(Resource, Debug, Default)]
pub struct Ocean {
    height: Height,
}

impl Ocean {
    /// The current height of the ocean.
    pub(crate) fn height(&self) -> Height {
        self.height
    }
}

/// Controls the ebb and flow of the tides, raising and lowering the ocean level.
pub(super) fn tides(
    mut ocean: ResMut<Ocean>,
    in_game_time: Res<InGameTime>,
    water_config: Res<WaterConfig>,
) {
    let time = in_game_time.elapsed_days();
    let settings = water_config.tide_settings;

    // The factor of TAU compensates for the natural period of the sine function.
    let scaled_time = time * std::f32::consts::TAU / settings.period.0;

    // The sine function can have a range of [-1, 1],
    // so at its lowest point we are subtracting the amplitude.
    // To ensure that the lowest point is at the minimum water level,
    // we add the minimum water level to the amplitude before applying the sine component.
    let tide_height =
        settings.minimum + settings.amplitude + settings.amplitude * scaled_time.sin();
    ocean.height = tide_height;
}
