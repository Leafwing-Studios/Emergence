//! Controls how the atmosphere and sky look.

use bevy::prelude::*;

use crate::{
    graphics::palette::environment::{MIDDAY_LIGHTNESS, SKY_SUNNY},
    simulation::light::TotalLight,
};

/// Logic and resources to modify the sky and atmosphere.
pub(super) struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(SKY_SUNNY))
            .add_system(animate_sky_color);
    }
}

/// Changes the `ClearColor` resource which drives the sky color based on the illuminance from the Sun.
fn animate_sky_color(mut clear_color: ResMut<ClearColor>, total_light: Res<TotalLight>) {
    let [hue, saturation, _, alpha] = clear_color.0.as_hsla_f32();

    // The midday lightness is the ideal lightness at noon.
    let lightness = total_light.normalized_illuminance().0 * MIDDAY_LIGHTNESS;

    let new_sky = Color::Hsla {
        hue,
        saturation,
        lightness,
        alpha,
    };

    clear_color.0 = new_sky;
}
