//! Logic and data types for energy.

use bevy::prelude::*;
use core::fmt::Display;
use std::ops::Range;

use crate::simulation::time::InGameTime;

/// The conditions for activity of an organism.
/// Organism can perform its regular actions only if all conditions are fulfilled.
#[derive(Debug, Clone, Copy, PartialEq, Component, Resource)]
pub(crate) struct ActivityConditions {
    /// Lower boud of `fraction_of_day` for activity
    fraction_of_day_start: f32,
    /// Upper boud of `fraction_of_day` for activity
    fraction_of_day_end: f32,
}

impl Default for ActivityConditions {
    fn default() -> Self {
        Self {
            fraction_of_day_start: f32::MIN,
            fraction_of_day_end: f32::MAX,
        }
    }
}

impl ActivityConditions {
    /// Quickly construct a new empty energy pool with a max energy of `max` and no regeneration.
    pub(crate) fn new(fraction_of_day: Range<f32>) -> Self {
        Self {
            fraction_of_day_start: fraction_of_day.start,
            fraction_of_day_end: fraction_of_day.end,
        }
    }

    /// Can this organism perform activities?
    pub(crate) fn is_active(&self, time: &InGameTime) -> bool {
        let fraction_of_day = time.fraction_of_day();
        fraction_of_day >= self.fraction_of_day_start && fraction_of_day <= self.fraction_of_day_end
    }
}

impl Display for ActivityConditions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}",
            self.fraction_of_day_start, self.fraction_of_day_end
        )
    }
}
