//! Controls how the atmosphere and sky look.

use bevy::prelude::*;

use crate::graphics::palette::environment::{MIDDAY_LIGHTNESS, SKY_SUNNY};

use super::lighting::{CelestialBody, Sun};

/// Logic and resources to modify the sky and atmosphere.
pub(super) struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(SKY_SUNNY))
            .add_system(animate_sky_color);
    }
}

/// Changes the ClearColor resource which drives the sky color based on the illuminance from the Sun.
fn animate_sky_color(
    mut clear_color: ResMut<ClearColor>,
    query: Query<(&CelestialBody, &Sun), Changed<CelestialBody>>,
) {
    for (celestial_body, _) in query.iter() {
        let max_illuminance = celestial_body.compute_max_light();
        let current_illuminance = celestial_body.compute_light();

        let [hue, saturation, _, alpha] = clear_color.0.as_hsla_f32();

        // The midday lightness is the ideal lightness at noon.
        //
        // Scaling up the max illuminance by the midday lightness ensures
        // we reach the max_illuminance at the same time we reach midday lightness.
        let target_scaled_max_illuminance = max_illuminance.0 / MIDDAY_LIGHTNESS;

        // Calculate a percentage of lightness of the given max illuminance
        let lightness = current_illuminance.0 / target_scaled_max_illuminance;

        let new_sky = Color::Hsla {
            hue,
            saturation,
            lightness,
            alpha,
        };

        clear_color.0 = new_sky;
    }
}
