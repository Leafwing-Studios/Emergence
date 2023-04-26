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
    asset_management::{manifest::Id, AssetState},
    enum_iter::IterableEnum,
    graphics::palette::infovis::{WATER_TABLE_COLOR_HIGH, WATER_TABLE_COLOR_LOW},
    player_interaction::{selection::ObjectInteraction, InteractionSystem},
    signals::{SignalKind, SignalStrength, SignalType, Signals},
    simulation::geometry::{MapGeometry, TilePos},
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
    units::unit_manifest::Unit,
    water::{DepthToWaterTable, WaterTable},
};

/// Systems and reources for communicating the state of the world to the player.
pub struct InfoVisPlugin;

impl Plugin for InfoVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(census)
            .init_resource::<Census>()
            .init_resource::<TileOverlay>()
            .add_systems(
                (
                    set_overlay_material,
                    display_tile_overlay
                        .after(InteractionSystem::SelectTiles)
                        .after(set_overlay_material),
                )
                    .distributive_run_if(in_state(AssetState::FullyLoaded)),
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
    pub(crate) overlay_type: OverlayType,
    /// The materials used to visualize the signal strength.
    ///
    /// Note that we cannot simply store a `Vec<Color>` here,
    /// because we need to be able to display the entire gradients of signal strength simultaneously.
    color_ramps: HashMap<SignalKind, Vec<Handle<StandardMaterial>>>,
    /// The materials used to visualize the distance to the water table.
    water_table_color_ramp: Vec<Handle<StandardMaterial>>,
    /// The images to be used to display the gradient in order to create a legend.
    legends: HashMap<SignalKind, Handle<Image>>,
    /// The image used to display the gradient for the water table.
    water_table_legend: Handle<Image>,
}

/// The type of information that is being visualized by the overlay.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OverlayType {
    /// No signal is being visualized.
    #[default]
    None,
    /// The signal strength of a single signal type is being visualized.
    Single(SignalType),
    /// The strongest signal in each cell is being visualized.
    StrongestSignal,
    /// The distance to the water table is being visualized.
    WaterTable,
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

        // Signals
        for kind in SignalKind::variants() {
            let material_assets: &mut Assets<StandardMaterial> =
                &mut world.resource_mut::<Assets<StandardMaterial>>();

            let colors =
                generate_color_gradient(kind.color_low(), kind.color_high(), Self::N_COLORS);
            let color_ramp = generate_color_ramp(&colors, material_assets);
            color_ramps.insert(kind, color_ramp);

            let legend_image = generate_legend(&colors, Self::LEGEND_WIDTH);
            let mut image_assets = world.resource_mut::<Assets<Image>>();
            let legend_image_handle = image_assets.add(legend_image);
            legends.insert(kind, legend_image_handle);
        }

        // Water table
        let water_table_colors = generate_color_gradient(
            WATER_TABLE_COLOR_LOW,
            WATER_TABLE_COLOR_HIGH,
            Self::N_COLORS,
        );
        let material_assets: &mut Assets<StandardMaterial> =
            &mut world.resource_mut::<Assets<StandardMaterial>>();
        let water_table_color_ramp = generate_color_ramp(&water_table_colors, material_assets);
        let water_table_legend_image = generate_legend(&water_table_colors, Self::LEGEND_WIDTH);
        let mut image_assets = world.resource_mut::<Assets<Image>>();
        let water_table_legend = image_assets.add(water_table_legend_image);

        Self {
            overlay_type: OverlayType::None,
            color_ramps,
            water_table_color_ramp,
            legends,
            water_table_legend,
        }
    }
}

/// Create a linearly interpolated color gradient between the two given colors.
fn generate_color_gradient(color_low: Color, color_high: Color, n_steps: usize) -> Vec<Color> {
    let mut colors = Vec::with_capacity(n_steps);
    for i in 0..n_steps {
        // Linearly interpolate the colors in the color ramp between SIGNAL_OVERLAY_LOW and SIGNAL_OVERLAY_HIGH
        // Make sure to use HSLA colorspace to avoid weird artifacts
        let t = i as f32 / (n_steps - 1) as f32;
        let Color::Hsla { hue: low_hue, saturation: low_saturation, lightness: low_lightness, alpha: low_alpha } = color_low else {
         panic!("Expected HSLA color for `color_low`");
    };

        let Color::Hsla { hue: high_hue, saturation: high_saturation, lightness: high_lightness, alpha: high_alpha } = color_high else {
        panic!("Expected HSLA color for `color_high`");
    };

        let hue = low_hue * (1.0 - t) + high_hue * t;
        let saturation = low_saturation * (1.0 - t) + high_saturation * t;
        let lightness = low_lightness * (1.0 - t) + high_lightness * t;
        let alpha = low_alpha * (1.0 - t) + high_alpha * t;

        let color = Color::hsla(hue, saturation, lightness, alpha);
        colors.push(color);
    }
    colors
}

/// Generates a color ramp of [`StandardMaterial`]s based on the given color gradient.
fn generate_color_ramp(
    colors: &Vec<Color>,
    material_assets: &mut Assets<StandardMaterial>,
) -> Vec<Handle<StandardMaterial>> {
    let mut color_ramp = Vec::with_capacity(colors.len());
    for color in colors {
        color_ramp.push(material_assets.add(StandardMaterial {
            base_color: *color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }));
    }

    color_ramp
}

/// Generates a legend image for the given color gradient.
#[allow(clippy::identity_op)]
fn generate_legend(colors: &Vec<Color>, legend_width: u32) -> Image {
    // Create the legend image
    let size = Extent3d {
        width: legend_width,
        height: colors.len() as u32,
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
    for (row, color) in colors.iter().rev().enumerate() {
        let row_start = row * size.width as usize * 4;
        for column in 0..size.width as usize {
            let pixel_start = row_start + column * 4;
            data[pixel_start + 0] = (color.r() * 255.0) as u8;
            data[pixel_start + 1] = (color.g() * 255.0) as u8;
            data[pixel_start + 2] = (color.b() * 255.0) as u8;
            data[pixel_start + 3] = (color.a() * 255.0) as u8;
        }
    }

    Image::new(size, dimension, data, format)
}

impl TileOverlay {
    /// The number of colors in the color ramp.
    pub(crate) const N_COLORS: usize = 16;

    /// The maximum displayed value for signal strength.
    const MAX_SIGNAL_STRENGTH: f32 = 1e3;

    /// The maximum displayed depth to the water table.
    ///
    /// Below this level, the water table is considered to be equally deep.
    const MAX_DEPTH_TO_WATER_TABLE: f32 = 5.;

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

    /// Gets the material that should be used to visualize the depth to the water table.
    ///
    /// If this is `None`, then the tile is covered with surface water.
    fn get_water_table_material(
        &self,
        depth_to_water_table: DepthToWaterTable,
    ) -> Option<Handle<StandardMaterial>> {
        let normalized_depth = match depth_to_water_table {
            DepthToWaterTable::Dry => 1.,
            DepthToWaterTable::Depth(depth) => {
                depth.0.min(Self::MAX_DEPTH_TO_WATER_TABLE) / Self::MAX_DEPTH_TO_WATER_TABLE
            }
            DepthToWaterTable::Flooded => return None,
        };

        let color_index: usize = (normalized_depth * (Self::N_COLORS as f32)) as usize;
        // Avoid indexing out of bounds by clamping to the maximum value in the case of extremely strong signals
        let color_index = color_index.min(Self::N_COLORS - 1);
        Some(self.water_table_color_ramp[color_index].clone_weak())
    }

    /// Gets the handle to the image that should be used to display the legend.
    pub(crate) fn legend_image_handle(&self, signal_kind: SignalKind) -> Handle<Image> {
        self.legends[&signal_kind].clone_weak()
    }

    /// Gets the handle to the material that should be used to display the legend for the water table.
    pub(crate) fn water_table_legend_image_handle(&self) -> Handle<Image> {
        self.water_table_legend.clone_weak()
    }
}

/// Sets the material for the currently visualized map overlay.
fn set_overlay_material(
    terrain_query: Query<(&TilePos, &Children), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    signals: Res<Signals>,
    water_table: Res<WaterTable>,
    map_geometry: Res<MapGeometry>,
    tile_overlay: Res<TileOverlay>,
) {
    if tile_overlay.overlay_type == OverlayType::None {
        return;
    }

    for (&tile_pos, children) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        if let Ok((mut overlay_material, mut overlay_visibility)) =
            overlay_query.get_mut(overlay_entity)
        {
            let maybe_material = match tile_overlay.overlay_type {
                OverlayType::None => None,
                OverlayType::Single(signal_type) => {
                    let signal_strength = signals.get(signal_type, tile_pos);
                    let signal_kind = signal_type.into();
                    tile_overlay.get_material(signal_kind, signal_strength)
                }
                OverlayType::StrongestSignal => signals
                    .strongest_goal_signal_at_position(tile_pos)
                    .and_then(|(signal_type, signal_strength)| {
                        let signal_kind = signal_type.into();
                        tile_overlay.get_material(signal_kind, signal_strength)
                    }),
                OverlayType::WaterTable => {
                    let depth_to_water_table =
                        water_table.depth_to_water_table(tile_pos, &map_geometry);
                    tile_overlay.get_water_table_material(depth_to_water_table)
                }
            };

            match maybe_material {
                Some(material) => {
                    *overlay_visibility = Visibility::Visible;
                    *overlay_material = material;
                }
                None => {
                    *overlay_visibility = Visibility::Hidden;
                }
            };
        } else {
            error!("Could not get overlay entity for tile {tile_pos:?}");
        }
    }
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
                if tile_overlay.overlay_type.is_none() {
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
