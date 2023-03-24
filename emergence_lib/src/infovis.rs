//! Computes and displays helpful information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use bevy::prelude::*;
use core::fmt::Display;

use crate::{
    asset_management::manifest::{Id, Terrain, Unit},
    signals::{SignalType, Signals},
    simulation::geometry::TilePos,
};

/// Systems and reources for communicating the state of the world to the player.
pub struct InfoVisPlugin;

impl Plugin for InfoVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(census)
            .init_resource::<Census>()
            .init_resource::<DebugColorScheme>()
            .insert_resource(DebugDisplayedSignal(SignalType::Unit(Id::from_name("ant"))))
            .add_system(visualize_signals);
    }
}

/// Tracks the population of organisms
#[derive(Debug, Resource, Default)]
pub(crate) struct Census {
    /// The total number of units of any kind
    total_units: usize,
}

impl Display for Census {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Population: {}", self.total_units)
    }
}

/// Counts the number of organisms
fn census(mut census: ResMut<Census>, unit_query: Query<(), With<Id<Unit>>>) {
    census.total_units = unit_query.iter().len();
}

/// The type of signal that is currently being visualized.
#[derive(Resource, Debug)]
pub(crate) struct DebugDisplayedSignal(SignalType);

/// The color ramp used to visualize signal strength.
#[derive(Resource, Debug)]
struct DebugColorScheme(Vec<Handle<StandardMaterial>>);

impl FromWorld for DebugColorScheme {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        let mut color_scheme = Vec::with_capacity(256);
        // FIXME: This color palette is not very colorblind-friendly, even though it was inspired
        // by matlab's veridis
        for i in 0..256 {
            let s = i as f32 / 255.0;
            color_scheme.push(material_assets.add(StandardMaterial {
                base_color: Color::Rgba {
                    red: 0.8 * (2.0 * s - s * s),
                    green: 0.8 * s.sqrt(),
                    blue: s * s * 0.6,
                    alpha: 0.8,
                },
                unlit: true,
                alpha_mode: AlphaMode::Add,
                ..Default::default()
            }));
        }

        color_scheme.shrink_to_fit();
        DebugColorScheme(color_scheme)
    }
}

/// Displays the currently visualized signal for the player using a map overlay.
fn visualize_signals(
    terrain_query: Query<(&TilePos, &Children), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    signals: Res<Signals>,
    displayed_signal: Res<DebugDisplayedSignal>,
    color_scheme: Res<DebugColorScheme>,
) {
    for (tile_pos, children) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        let (mut overlay_material, mut overlay_visibility) =
            overlay_query.get_mut(overlay_entity).unwrap();

        let signal_strength = signals.get(displayed_signal.0, *tile_pos).value();
        // The scale is logarithmic, so that small nuances are still pretty visible
        let scaled_strength = signal_strength.ln_1p() / 6.0;
        let color_index = if signal_strength < f32::EPSILON {
            *overlay_visibility = Visibility::Hidden;
            continue;
        } else {
            *overlay_visibility = Visibility::Visible;
            ((scaled_strength * 254.0) as usize) + 1
        };
        *overlay_material.as_mut() = color_scheme.0[color_index.min(255)].clone_weak();
    }
}
