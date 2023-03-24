//! Computes and displays helpful information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
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
    /// The image to be used to display the gradient in order to create a legend.
    legend_image_handle: Handle<Image>,
}

impl FromWorld for TileOverlay {
    fn from_world(world: &mut World) -> Self {
        let mut colors = Vec::with_capacity(Self::N_COLORS);
        for i in 0..Self::N_COLORS {
            let s = i as f32 / (Self::N_COLORS as f32 - 1.0);
            colors.push(Color::Rgba {
                red: 0.8 * (2.0 * s - s * s),
                green: 0.8 * s.sqrt(),
                blue: s * s * 0.6,
                alpha: 0.8,
            });
        }

        // FIXME: This color palette is not very colorblind-friendly, even though it was inspired
        // by matlab's veridis

        let mut color_ramp = Vec::with_capacity(Self::N_COLORS);
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        for base_color in colors.iter().cloned() {
            color_ramp.push(material_assets.add(StandardMaterial {
                base_color,
                unlit: true,
                alpha_mode: AlphaMode::Add,
                ..Default::default()
            }));
        }
        color_ramp.shrink_to_fit();

        // Create the legend image
        let size = Extent3d {
            width: Self::LEGEND_WIDTH,
            height: Self::N_COLORS as u32,
            depth_or_array_layers: 1,
        };
        let dimension = TextureDimension::D2;
        let format = TextureFormat::Rgba8UnormSrgb;

        // Initialize the legend data with all zeros
        let mut data = vec![0; size.width as usize * size.height as usize * 4];

        // Set the color of each pixel to the corresponding color in the color ramp
        // Each line is a row of pixels of the same color, corresponding to a value in the color ramp
        // Each pixel is represented by 4 bytes, in RGBA order
        for (row, color) in colors.into_iter().enumerate() {
            let row_start = row * size.width as usize * 4;
            for column in 0..size.width as usize {
                let pixel_start = row_start + column * 4;
                data[pixel_start + 0] = (color.r() * 255.0) as u8;
                data[pixel_start + 1] = (color.g() * 255.0) as u8;
                data[pixel_start + 2] = (color.b() * 255.0) as u8;
                data[pixel_start + 3] = (color.a() * 255.0) as u8;
            }
        }

        let legend_image = Image::new(size, dimension, data, format);

        let mut image_assets = world.resource_mut::<Assets<Image>>();
        let legend_image_handle = image_assets.add(legend_image);

        Self {
            visualized_signal: None,
            color_ramp,
            legend_image_handle,
        }
    }
}

impl TileOverlay {
    /// The number of colors in the color ramp.
    pub(crate) const N_COLORS: usize = 256;

    /// The maximum displayed value for signal strength.
    const MAX_SIGNAL_STRENGTH: f32 = 1e3;

    /// The width of the legend image.
    pub(crate) const LEGEND_WIDTH: u32 = 32;

    /// Gets the material that should be used to visualize the given signal strength, if any.
    ///
    /// If this is `None`, then the signal strength is too weak to be visualized and the tile should be invisible.
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

    /// Gets the handle to the image that should be used to display the legend.
    pub(crate) fn legend_image_handle(&self) -> Handle<Image> {
        self.legend_image_handle.clone_weak()
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
