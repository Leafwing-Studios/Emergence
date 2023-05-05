//! Computes the amount of light available at a given time.

use bevy::prelude::*;
use core::fmt::Display;
use derive_more::{Add, AddAssign};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

use crate::{
    graphics::lighting::{CelestialBody, Sun},
    simulation::SimulationSet,
};

use self::shade::{compute_received_light, compute_shade};

pub(crate) mod shade;

/// Systems and resources for computing light (in in-game quantities).
pub(super) struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TotalLight>().add_systems(
            (compute_light, compute_shade, compute_received_light)
                .chain()
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// The total current amount of light available.
#[derive(Resource, Default, Debug)]
pub(crate) struct TotalLight {
    /// The total amount of light available, in lux.
    illuminance: Illuminance,
    /// The maximum expected amount of light available, in lux.
    max_illuminance: Illuminance,
}

impl TotalLight {
    /// The normalized amount of light available.
    ///
    /// This is expected to be 0 in pitch black darkness, and 1 in full daylight.
    pub(crate) fn normalized_illuminance(&self) -> NormalizedIlluminance {
        NormalizedIlluminance(self.illuminance.0 / self.max_illuminance.0)
    }
}

impl Display for TotalLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.normalized_illuminance())
    }
}

/// A normalized amount of light.
///
/// This is expected to be 0 in pitch black darkness, and 1 in full daylight.
#[derive(Default, PartialEq, PartialOrd, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NormalizedIlluminance(pub f32);

impl Display for NormalizedIlluminance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.)
    }
}

impl Mul<f32> for NormalizedIlluminance {
    type Output = NormalizedIlluminance;

    fn mul(self, rhs: f32) -> Self::Output {
        NormalizedIlluminance(self.0 * rhs)
    }
}

/// Light illuminance in lux.
#[derive(
    Add, AddAssign, Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct Illuminance(pub f32);

impl Display for Illuminance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Rounds to the nearest 100 lux
        let rounded_illuminance = (self.0 / 100.).round() * 100.;

        write!(f, "{rounded_illuminance:.0} lux")
    }
}

impl Mul<f32> for Illuminance {
    type Output = Illuminance;

    fn mul(self, rhs: f32) -> Self::Output {
        Illuminance(self.0 * rhs)
    }
}

impl Mul<Illuminance> for f32 {
    type Output = Illuminance;

    fn mul(self, rhs: Illuminance) -> Self::Output {
        Illuminance(self * rhs.0)
    }
}

/// Computes the amount of light available from each celestial body based on its position in the sky and luminous intensity.
fn compute_light(
    mut query: Query<(&CelestialBody, &Visibility)>,
    primary_body_query: Query<&CelestialBody, With<Sun>>,
    mut total_light: ResMut<TotalLight>,
) {
    if total_light.max_illuminance == Illuminance(0.0) {
        if let Ok(primary_body) = primary_body_query.get_single() {
            total_light.max_illuminance = primary_body.compute_max_light();
        }
    }

    let mut sum = Illuminance(0.0);
    for (body, visibility) in query.iter_mut() {
        if visibility == Visibility::Visible {
            let light = body.compute_light();
            sum += light;
        }
    }
    total_light.illuminance = sum;
}
