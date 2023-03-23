//! Computes the amount of light available at a given time.

use bevy::prelude::*;
use core::fmt::Display;

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
struct TotalLight {
    illuminance: f32,
}

impl Display for TotalLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} lux", self.illuminance)
    }
}

/// Computes the amount of light available from each celestial body based on its position in the sky and luminous intensity.
fn compute_light(mut query: Query<&CelestialBody>, mut total_light: ResMut<TotalLight>) {
    let mut sum = 0.0;
    for body in query.iter_mut() {
        let light = body.compute_light();
        sum += light;
    }
    total_light.illuminance = sum;
}
