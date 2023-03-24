//! Computes and displays helpful information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use bevy::prelude::*;
use core::fmt::Display;

use crate::{
    asset_management::{
        manifest::{Id, Terrain, Unit},
        terrain::TerrainHandles,
    },
    player_interaction::{selection::ObjectInteraction, InteractionSystem},
    signals::{SignalStrength, SignalType, Signals},
    simulation::geometry::TilePos,
};

/// Systems and reources for communicating the state of the world to the player.
pub struct InfoVisPlugin;

impl Plugin for InfoVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(census)
            .init_resource::<Census>()
            .init_resource::<TileOverlay>()
            .add_system(visualize_signals);
        //.add_system(display_tile_overlay.after(InteractionSystem::SelectTiles));
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

/// Controls the display of the tile overlay.
#[derive(Resource, Debug)]
pub(crate) struct TileOverlay {
    /// The type of signal that is currently being visualized.
    pub(crate) visualized_signal: Option<SignalType>,
    /// The materials used to visualize the signal strength.
    ///
    /// Note that we cannot simply store a `Vec<Color>` here,
    /// because we need to be able to display the entire gradients of signal strength simultaneously.
    color_ramp: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for TileOverlay {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        let mut color_ramp = Vec::with_capacity(256);
        // FIXME: This color palette is not very colorblind-friendly, even though it was inspired
        // by matlab's veridis
        for i in 0..256 {
            let s = i as f32 / 255.0;
            color_ramp.push(material_assets.add(StandardMaterial {
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

        color_ramp.shrink_to_fit();

        Self {
            visualized_signal: None,
            color_ramp,
        }
    }
}

impl TileOverlay {
    /// The number of colors in the color ramp.
    const N_COLORS: usize = 256;

    /// The maximum displayed value for signal strength.
    const MAX_SIGNAL_STRENGTH: f32 = 1e5;

    fn get_material(&self, signal_strength: SignalStrength) -> Option<Handle<StandardMaterial>> {
        // Don't bother visualizing signals that are too weak to be detected
        if signal_strength.value() < f32::EPSILON {
            return None;
        }

        // At MAX_SIGNAL_STRENGTH, we want to fetch the last color in the ramp.
        // At 0, we want to fetch the first color in the ramp.
        // We can achieve this by scaling the logged signal strength by the number of colors in the ramp.
        let logged_strength_at_max = Self::MAX_SIGNAL_STRENGTH.ln_1p();
        // The logged_strength_at_min is equal to 0, as we are taking the log of (0+1)

        // Now, we can compute the scaling factor as (logged_strength_at_max - logged_strength_at_min) / N_COLORS
        let scaling_factor = logged_strength_at_max / Self::N_COLORS as f32;

        // The scale is logarithmic, so that small nuances are still pretty visible
        // By adding 1 to the signal strength, we avoid taking the log of 0
        let scaled_strength = signal_strength.value() / scaling_factor;
        let logged_strength = scaled_strength.ln_1p();

        // Scale the signal strength to the number of colors in the ramp
        let color_index: usize = (logged_strength * Self::N_COLORS as f32) as usize;
        // Avoid indexing out of bounds by clamping to the maximum value in the case of extremely strong signals
        let color_index = color_index.min(Self::N_COLORS - 1);
        Some(self.color_ramp[color_index].clone_weak())
    }
}

/// Displays the currently visualized signal for the player using a map overlay.
fn visualize_signals(
    terrain_query: Query<(&TilePos, &Children), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    signals: Res<Signals>,
    tile_overlay: Res<TileOverlay>,
) {
    if let Some(signal_type) = tile_overlay.visualized_signal {
        for (tile_pos, children) in terrain_query.iter() {
            // This is promised to be the correct entity in the initialization of the terrain's children
            let overlay_entity = children[1];

            if let Ok((mut overlay_material, mut overlay_visibility)) =
                overlay_query.get_mut(overlay_entity)
            {
                let signal_strength = signals.get(signal_type, *tile_pos);
                let maybe_material = tile_overlay.get_material(signal_strength);
                match maybe_material {
                    Some(material) => {
                        *overlay_visibility = Visibility::Visible;
                        *overlay_material = material;
                    }
                    None => {
                        *overlay_visibility = Visibility::Hidden;
                    }
                }
            } else {
                error!("Could not get overlay entity for tile {tile_pos:?}");
            }
        }
    }
}

/// Displays the overlay of the tile
fn display_tile_overlay(
    terrain_query: Query<
        (&Children, &ObjectInteraction),
        (With<Id<Terrain>>, Changed<ObjectInteraction>),
    >,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    terrain_handles: Res<TerrainHandles>,
) {
    for (children, object_interaction) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        let (mut overlay_material, mut overlay_visibility) =
            overlay_query.get_mut(overlay_entity).unwrap();

        match object_interaction {
            ObjectInteraction::None => {
                *overlay_visibility = Visibility::Hidden;
            }
            _ => {
                *overlay_visibility = Visibility::Visible;
                let new_material = terrain_handles
                    .interaction_materials
                    .get(object_interaction)
                    .unwrap()
                    .clone_weak();

                *overlay_material = new_material;
            }
        }
    }
}
