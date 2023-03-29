//! Computes and displays helpful information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use core::fmt::Display;

use crate::{
    asset_management::{
        manifest::{Id, Terrain, Unit},
        terrain::TerrainHandles,
    },
    enum_iter::IterableEnum,
    player_interaction::{selection::ObjectInteraction, InteractionSystem},
    signals::{SignalKind, SignalStrength, SignalType, Signals},
    simulation::geometry::TilePos,
};

/// Systems and reources for communicating the state of the world to the player.
pub struct InfoVisPlugin;

impl Plugin for InfoVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(census)
            .init_resource::<Census>()
            .init_resource::<TileOverlay>()
            .add_system(set_overlay_material)
            .add_system(
                display_tile_overlay
                    .after(InteractionSystem::SelectTiles)
                    .after(set_overlay_material),
            );
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
    pub(crate) visualized_signal: OverlayType,
    /// The materials used to visualize the signal strength.
    ///
    /// Note that we cannot simply store a `Vec<Color>` here,
    /// because we need to be able to display the entire gradients of signal strength simultaneously.
    color_ramps: HashMap<SignalKind, Vec<Handle<StandardMaterial>>>,
    /// The images to be used to display the gradient in order to create a legend.
    legends: HashMap<SignalKind, Handle<Image>>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OverlayType {
    /// No signal is being visualized.
    #[default]
    None,
    /// The signal strength of a single signal type is being visualized.
    Single(SignalType),
    /// The strongest signal in each cell is being visualized.
    StrongestSignal,
}

impl OverlayType {
    /// Returns true if this overlay type is `None`.
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl From<Option<SignalType>> for OverlayType {
    fn from(signal_type: Option<SignalType>) -> Self {
        match signal_type {
            Some(signal_type) => Self::Single(signal_type),
            None => Self::None,
        }
    }
}

impl FromWorld for TileOverlay {
    #[allow(clippy::identity_op)]
    fn from_world(world: &mut World) -> Self {
        let mut color_ramps = HashMap::new();
        let mut legends = HashMap::new();

        for kind in SignalKind::variants() {
            let mut colors = Vec::with_capacity(Self::N_COLORS);
            for i in 0..Self::N_COLORS {
                // Linearly interpolate the colors in the color ramp between SIGNAL_OVERLAY_LOW and SIGNAL_OVERLAY_HIGH
                // Make sure to use HSLA colorspace to avoid weird artifacts
                let t = i as f32 / (Self::N_COLORS - 1) as f32;
                let Color::Hsla { hue: low_hue, saturation: low_saturation, lightness: low_lightness, alpha: low_alpha } = kind.color_low() else {
                 panic!("Expected HSLA color for `color_low`");
            };

                let Color::Hsla { hue: high_hue, saturation: high_saturation, lightness: high_lightness, alpha: high_alpha } = kind.color_high() else {
                panic!("Expected HSLA color for `color_high`");
            };

                let hue = low_hue * (1.0 - t) + high_hue * t;
                let saturation = low_saturation * (1.0 - t) + high_saturation * t;
                let lightness = low_lightness * (1.0 - t) + high_lightness * t;
                let alpha = low_alpha * (1.0 - t) + high_alpha * t;

                let color = Color::hsla(hue, saturation, lightness, alpha);
                colors.push(color);
            }
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
            color_ramps.insert(kind, color_ramp);

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
            // We need to reverse the order of the rows, because the image is stored in memory from top to bottom
            // and we want the lowest value to be at the bottom of the image.
            for (row, color) in colors.into_iter().rev().enumerate() {
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
            legends.insert(kind, legend_image_handle);
        }

        Self {
            visualized_signal: OverlayType::None,
            color_ramps,
            legends,
        }
    }
}

impl TileOverlay {
    /// The number of colors in the color ramp.
    pub(crate) const N_COLORS: usize = 16;

    /// The maximum displayed value for signal strength.
    const MAX_SIGNAL_STRENGTH: f32 = 1e2;

    /// The width of the legend image.
    pub(crate) const LEGEND_WIDTH: u32 = 32;

    /// Gets the material that should be used to visualize the given signal strength, if any.
    ///
    /// If this is `None`, then the signal strength is too weak to be visualized and the tile should be invisible.
    fn get_material(
        &self,
        signal_kind: SignalKind,
        signal_strength: SignalStrength,
    ) -> Option<Handle<StandardMaterial>> {
        // Don't bother visualizing signals that are too weak to be detected
        if signal_strength.value() < f32::EPSILON {
            return None;
        }

        // At MAX_SIGNAL_STRENGTH, we want to fetch the last color in the ramp.
        // At 0, we want to fetch the first color in the ramp.

        // The scale is logarithmic, so that small nuances are still pretty visible
        // By adding 1 to the signal strength, we avoid taking the log of 0
        // This produces a value in the range [0, 1] for all signal strengths that we care about.
        let normalized_strength =
            signal_strength.value().ln_1p() / Self::MAX_SIGNAL_STRENGTH.ln_1p();

        // Now that strength is normalized, we can scale it to the number of colors in the ramp
        // Which should give us a nice distribution of colors that uses the entire range.
        let color_index: usize = (normalized_strength * (Self::N_COLORS as f32)) as usize;
        // Avoid indexing out of bounds by clamping to the maximum value in the case of extremely strong signals
        let color_index = color_index.min(Self::N_COLORS - 1);
        Some(self.color_ramps[&signal_kind][color_index].clone_weak())
    }

    /// Gets the handle to the image that should be used to display the legend.
    pub(crate) fn legend_image_handle(&self, signal_kind: SignalKind) -> Handle<Image> {
        self.legends[&signal_kind].clone_weak()
    }
}

/// Sets the material for the currently visualized map overlay.
fn set_overlay_material(
    terrain_query: Query<(&TilePos, &Children), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    signals: Res<Signals>,
    tile_overlay: Res<TileOverlay>,
) {
    match &tile_overlay.visualized_signal {
        OverlayType::None => (),
        OverlayType::Single(signal_type) => {
            for (tile_pos, children) in terrain_query.iter() {
                // This is promised to be the correct entity in the initialization of the terrain's children
                let overlay_entity = children[1];

                if let Ok((mut overlay_material, mut overlay_visibility)) =
                    overlay_query.get_mut(overlay_entity)
                {
                    let signal_strength = signals.get(*signal_type, *tile_pos);
                    let signal_kind = (*signal_type).into();
                    let maybe_material = tile_overlay.get_material(signal_kind, signal_strength);
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
        OverlayType::StrongestSignal => todo!(),
    };
}

/// Displays the overlay of the tile
fn display_tile_overlay(
    terrain_query: Query<(&Children, &ObjectInteraction), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    terrain_handles: Res<TerrainHandles>,
    tile_overlay: Res<TileOverlay>,
) {
    for (children, object_interaction) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        let (mut overlay_material, mut overlay_visibility) =
            overlay_query.get_mut(overlay_entity).unwrap();

        match object_interaction {
            ObjectInteraction::None => {
                if tile_overlay.visualized_signal.is_none() {
                    *overlay_visibility = Visibility::Hidden;
                }
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
