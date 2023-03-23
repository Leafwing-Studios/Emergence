//! Computes the amount of light available at a given time.

use bevy::prelude::*;
use core::fmt::Display;
use derive_more::{Add, AddAssign};

use super::SimulationSet;
use crate::graphics::lighting::CelestialBody;

/// Systems and resources for computing light (in in-game quantities).
pub(super) struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (compute_light,)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .init_resource::<TotalLight>();
    }
}

/// The total current amount of light available.
#[derive(Resource, Default, Debug)]
pub(crate) struct TotalLight {
    /// The total amount of light available, in lux.
    illuminance: Illuminance,
}

impl TotalLight {
    /// The total amount of light available, in lux.
    pub(crate) fn illuminance(&self) -> Illuminance {
        self.illuminance
    }
}

/// Light illuminance in lux.
#[derive(Add, AddAssign, Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Illuminance(pub(crate) f32);

impl Display for TotalLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Rounds to the nearest 100 lux
        let rounded_illuminance = (self.illuminance.0 / 100.).round() * 100.;

        write!(f, "{rounded_illuminance:.0} lux")
    }
}

/// Computes the amount of light available from each celestial body based on its position in the sky and luminous intensity.
fn compute_light(mut query: Query<&CelestialBody>, mut total_light: ResMut<TotalLight>) {
    let mut sum = Illuminance(0.0);
    for body in query.iter_mut() {
        let light = body.compute_light();
        sum += light;
    }
    total_light.illuminance = sum;
}
