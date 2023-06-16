//! Units should die of old age.
//!
//! As discussed in <https://github.com/Leafwing-Studios/Emergence/issues/704>,
//! this enables players to more easily control their population size and stockpile food.

use std::fmt::{Display, Formatter};

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::simulation::time::{Days, InGameTime};

/// The age of a unit, in in-game days.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Age {
    /// The current age of the unit.
    current: Days,
    /// The maximum age of the unit.
    max: Days,
}

impl Age {
    /// Creates a new [`Age`] with the given maximum age.
    pub fn newborn(max: Days) -> Self {
        Self {
            current: Days::ZERO,
            max,
        }
    }

    /// Creates a new [`Age`] with the given maximum age and a random current age.
    pub fn randomized(rng: &mut impl Rng, max: Days) -> Self {
        Self {
            current: Days(rng.gen::<f32>() * max.0),
            max,
        }
    }

    /// Returns the current age in [`Days`].
    pub fn current(&self) -> Days {
        self.current
    }

    /// Returns the maximum age in [`Days`].
    pub fn max(&self) -> Days {
        self.max
    }
}

impl Display for Age {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}/{:.2} days", self.current.0, self.max.0)
    }
}

/// Advances the age of all units by the elapsed time and kills them if they are too old.
pub(super) fn aging(
    mut commands: Commands,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
    mut query: Query<(&mut Age, Entity)>,
) {
    let delta_time = fixed_time.period.as_secs_f32();
    let delta_days = Days(delta_time / in_game_time.seconds_per_day());

    for (mut age, entity) in query.iter_mut() {
        age.current += delta_days;

        if age.current > age.max {
            commands.entity(entity).despawn_recursive();
        }
    }
}
