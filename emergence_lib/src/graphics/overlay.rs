//! Computes and displays helpful spatial information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use crate::{
    self as emergence_lib,
    geometry::Volume,
    graphics::palette::infovis::{NEUTRAL_INFOVIS_COLOR, OVERLAY_ALPHA},
    light::{shade::ReceivedLight, Illuminance},
    water::FlowVelocity,
};
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use emergence_macros::IterableEnum;

use crate::{
    asset_management::manifest::Id,
    enum_iter::IterableEnum,
    geometry::{Height, MapGeometry, VoxelPos},
    graphics::palette::infovis::{WATER_TABLE_COLOR_HIGH, WATER_TABLE_COLOR_LOW},
    player_interaction::{selection::ObjectInteraction, InteractionSystem},
    signals::{SignalKind, SignalStrength, SignalType, Signals},
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
    water::{PreviousWaterVolume, WaterDepth, WaterVolume},
};

use super::GraphicsSet;

/// Systems and reources for communicating the state of the world to the player.
pub(super) struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOverlay>()
            .add_systems(
                (
                    set_overlay_material,
                    set_overlay_height,
                    display_player_selection
                        .after(InteractionSystem::SelectTiles)
                        .after(set_overlay_material),
                )
                    .in_set(GraphicsSet),
            )
            .add_system(
                spawn_overlay_entities
                    .in_base_set(CoreSet::PreUpdate)
                    .in_set(GraphicsSet),
            );
    }
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
    signal_color_ramps: HashMap<SignalKind, Vec<Handle<StandardMaterial>>>,
    /// The materials used to visualize the distance to the water table.
    water_table_color_ramp: Vec<Handle<StandardMaterial>>,
    /// The materials used to visualize light levels
    light_level_color_ramp: HashMap<Illuminance, Handle<StandardMaterial>>,
    /// The materials used to visualize the net change in water volume.
    flux_color_ramp: Vec<Handle<StandardMaterial>>,
    /// The materials used to visualize vector fields.
    vector_field_materials: HashMap<DiscretizedVector, Handle<StandardMaterial>>,
    /// The images to be used to display the gradient in order to create a legend.
    signal_legends: HashMap<SignalKind, Handle<Image>>,
    /// The image used to display the gradient for the water table.
    water_table_legend: Handle<Image>,
    /// The image used to display the gradient for the net change in water volume.
    flux_legend: Handle<Image>,
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
    DepthToWaterTable,
    /// The height of the water table is being visualized.
    HeightOfWaterTable,
    /// The flow velocity of the water table is being visualized.
    VelocityOfWaterTable,
    /// The net increase or decrease in water volume is being visualized.
    NetWater,
    /// Shows the current light level of each tile.
    LightLevel,
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

        // Flux
        let flux_colors = generate_color_bigradient(
            WATER_TABLE_COLOR_HIGH,
            WATER_TABLE_COLOR_LOW,
            Self::N_COLORS,
        );
        let material_assets: &mut Assets<StandardMaterial> =
            &mut world.resource_mut::<Assets<StandardMaterial>>();
        let flux_color_ramp = generate_color_ramp(&flux_colors, material_assets);
        let flux_legend_image = generate_legend(&flux_colors, Self::LEGEND_WIDTH);
        let mut image_assets = world.resource_mut::<Assets<Image>>();
        let flux_legend = image_assets.add(flux_legend_image);

        let material_assets: &mut Assets<StandardMaterial> =
            &mut world.resource_mut::<Assets<StandardMaterial>>();

        let mut light_level_color_ramp = HashMap::new();
        for variant in Illuminance::variants() {
            let base_color = variant.info_vis_color();
            let handle = material_assets.add(StandardMaterial {
                base_color,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            });

            light_level_color_ramp.insert(variant, handle);
        }

        // Vector fields
        let material_assets: &mut Assets<StandardMaterial> =
            &mut world.resource_mut::<Assets<StandardMaterial>>();
        let vector_field_materials = generate_vector_field_materials(material_assets);

        Self {
            overlay_type: OverlayType::None,
            signal_color_ramps: color_ramps,
            water_table_color_ramp,
            flux_color_ramp,
            light_level_color_ramp,
            vector_field_materials,
            signal_legends: legends,
            water_table_legend,
            flux_legend,
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

/// Create a linearly interpolated color gradient between the two given colors, with a neutral white in the middle.
fn generate_color_bigradient(color_low: Color, color_high: Color, n_steps: usize) -> Vec<Color> {
    let mut colors = Vec::with_capacity(n_steps);
    let half_way = n_steps / 2;

    // Generate the first half of the gradient
    colors.extend(generate_color_gradient(
        color_low,
        NEUTRAL_INFOVIS_COLOR,
        half_way,
    ));

    // Generate the second half of the gradient
    colors.extend(generate_color_gradient(
        NEUTRAL_INFOVIS_COLOR,
        color_high,
        n_steps - half_way,
    ));
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

/// Generates a set of [`StandardMaterial`]s for visualizing a vector field.
///
/// Hue is used for direction, saturation is used for magnitude.
fn generate_vector_field_materials(
    material_assets: &mut Assets<StandardMaterial>,
) -> HashMap<DiscretizedVector, Handle<StandardMaterial>> {
    let mut materials = HashMap::default();
    for direction in DiscretizedDirection::variants() {
        for magnitude in DiscretizedMagnitude::variants() {
            let discretized_vector = DiscretizedVector {
                direction,
                magnitude,
            };

            let hue = direction.degrees();
            let saturation = magnitude.saturation();
            let color = Color::hsla(hue, saturation, 0.5, OVERLAY_ALPHA);

            materials.insert(
                discretized_vector,
                material_assets.add(StandardMaterial {
                    base_color: color,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                }),
            );
        }
    }

    materials
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
    const MAX_DEPTH_TO_WATER_TABLE: f32 = 10.;

    /// The maximum volume of water per second of flux to be displayed.
    ///
    /// Above this volume, the water flux is considered to be equally large.
    const MAX_FLUX: Volume = Volume(1e-2);

    /// The width of the legend image.
    pub(crate) const LEGEND_WIDTH: u32 = 32;

    /// Gets the material that should be used to visualize the given signal strength, if any.
    ///
    /// If this is `None`, then the signal strength is too weak to be visualized and the tile should be invisible.
    fn get_signal_material(
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
        Some(self.signal_color_ramps[&signal_kind][color_index].clone_weak())
    }

    /// Gets the material that should be used to visualize the depth to the water table.
    ///
    /// If this is `None`, then the tile is covered with surface water.
    fn get_water_table_material(
        &self,
        depth_to_water_table: WaterDepth,
    ) -> Option<Handle<StandardMaterial>> {
        let normalized_depth = match depth_to_water_table {
            WaterDepth::Dry => 1.,
            WaterDepth::Underground(depth) => {
                depth.0.min(Self::MAX_DEPTH_TO_WATER_TABLE) / Self::MAX_DEPTH_TO_WATER_TABLE
            }
            WaterDepth::Flooded(..) => return None,
        };

        let color_index: usize = (normalized_depth * (Self::N_COLORS as f32)) as usize;
        // Avoid indexing out of bounds by clamping to the maximum value in the case of extremely strong signals
        let color_index = color_index.min(Self::N_COLORS - 1);
        Some(self.water_table_color_ramp[color_index].clone_weak())
    }

    /// Gets the material that should be used to visualize the flow of water with the provided `volume_per_second`.
    fn get_water_flux_material(&self, volume_per_second: Volume) -> Handle<StandardMaterial> {
        let normalized_volume = volume_per_second / Self::MAX_FLUX;
        // How far along the color ramp we should be
        // Divide by 2 then add 0.5 to shift the range from [-1, 1] to [0, 1]
        let color_value = normalized_volume / 2. + 0.5;
        let clamped_color_value = color_value.clamp(0., 1.);

        // Avoid indexing out of bounds by clamping to the maximum value in the case of extreme water volumes
        let color_index: usize = (clamped_color_value * Self::N_COLORS as f32) as usize;
        self.flux_color_ramp[color_index.min(Self::N_COLORS - 1)].clone_weak()
    }

    /// Gets the material that should be used to visualize the flow of water with the provided `flow_velocity`.
    pub(crate) fn get_flow_velocity_material(
        &self,
        flow_velocity: &FlowVelocity,
    ) -> Option<Handle<StandardMaterial>> {
        let magnitude = DiscretizedMagnitude::from_water_flow_volume(flow_velocity.magnitude());
        if magnitude == DiscretizedMagnitude::None {
            return None;
        }

        let direction = DiscretizedDirection::from_radians(flow_velocity.direction());
        let discretized_vector = DiscretizedVector {
            magnitude,
            direction,
        };

        Some(
            self.vector_field_materials
                .get(&discretized_vector)
                .unwrap()
                .clone_weak(),
        )
    }

    /// Gets the material that should be used to visualize the provided `received_light`
    pub(crate) fn get_light_level_material(
        &self,
        received_light: &ReceivedLight,
    ) -> Option<Handle<StandardMaterial>> {
        self.light_level_color_ramp
            .get(&received_light.0)
            .map(|material| material.clone_weak())
    }

    /// Gets the handle to the image that should be used to display the legend.
    pub(crate) fn signal_legend_image_handle(&self, signal_kind: SignalKind) -> Handle<Image> {
        self.signal_legends[&signal_kind].clone_weak()
    }

    /// Gets the handle to the material that should be used to display the legend for the water table.
    pub(crate) fn water_table_legend_image_handle(&self) -> Handle<Image> {
        self.water_table_legend.clone_weak()
    }

    /// Gets the handle to the material that should be used to display the legend for the water flux.
    pub(crate) fn flux_legend_image_handle(&self) -> Handle<Image> {
        self.flux_legend.clone_weak()
    }
}

/// Sets the material for the currently visualized map overlay.
fn set_overlay_material(
    mut overlay_query: Query<
        (&VoxelPos, &mut Handle<StandardMaterial>, &mut Visibility),
        With<Overlay>,
    >,
    terrain_query: Query<&ReceivedLight, With<Id<Terrain>>>,
    water_depth_query: Query<&WaterDepth>,
    water_volume_query: Query<(&WaterVolume, &PreviousWaterVolume)>,
    terrain_pos_query: Query<&VoxelPos, With<Id<Terrain>>>,
    flow_velocity_query: Query<&FlowVelocity>,
    signals: Res<Signals>,
    map_geometry: Res<MapGeometry>,
    tile_overlay: Res<TileOverlay>,
    fixed_time: Res<FixedTime>,
) {
    if tile_overlay.overlay_type == OverlayType::None {
        return;
    }

    for (&voxel_pos, mut overlay_material, mut overlay_visibility) in overlay_query.iter_mut() {
        let maybe_material = match tile_overlay.overlay_type {
            OverlayType::None => None,
            OverlayType::Single(signal_type) => {
                // We must look at the voxel above the terrain to get the signal strength, as those are the voxels that units can walk in
                let signal_strength = signals.get(signal_type, voxel_pos.above());
                let signal_kind = signal_type.into();
                tile_overlay.get_signal_material(signal_kind, signal_strength)
            }
            OverlayType::StrongestSignal => signals
                // We must look at the voxel above the terrain to get the signal strength, as those are the voxels that units can walk in
                .strongest_goal_signal_at_position(voxel_pos.above())
                .and_then(|(signal_type, signal_strength)| {
                    let signal_kind = signal_type.into();
                    tile_overlay.get_signal_material(signal_kind, signal_strength)
                }),
            OverlayType::DepthToWaterTable => {
                let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
                let water_depth = *water_depth_query.get(terrain_entity).unwrap();

                tile_overlay.get_water_table_material(water_depth)
            }
            OverlayType::HeightOfWaterTable => {
                let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
                let water_depth = *water_depth_query.get(terrain_entity).unwrap();
                let terrain_height = terrain_pos_query.get(terrain_entity).unwrap().height();
                let water_table_height = water_depth.water_table_height(terrain_height);

                // FIXME: use a dedicated color ramp for this, rather than hacking it
                // We subtract here to ensure that blue == wet and red == dry
                let inverted_height = WaterDepth::Underground(
                    Height(TileOverlay::MAX_DEPTH_TO_WATER_TABLE) - water_table_height,
                );

                tile_overlay.get_water_table_material(inverted_height)
            }
            OverlayType::VelocityOfWaterTable => {
                let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
                let flow_velocity = flow_velocity_query.get(terrain_entity).unwrap();

                tile_overlay.get_flow_velocity_material(flow_velocity)
            }
            OverlayType::NetWater => {
                let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
                let (current_water_volume, previous_water_volume) =
                    water_volume_query.get(terrain_entity).unwrap();

                let net_water = *current_water_volume - previous_water_volume.0;
                let volume_per_second = net_water.volume() / fixed_time.period.as_secs_f32();

                Some(tile_overlay.get_water_flux_material(volume_per_second))
            }
            OverlayType::LightLevel => {
                let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
                let received_light = terrain_query.get(terrain_entity).unwrap();

                tile_overlay.get_light_level_material(received_light)
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
    }
}

/// Sets the overlay of the tile based on the player's selection.
fn display_player_selection(
    terrain_query: Query<&ObjectInteraction, With<Id<Terrain>>>,
    mut overlay_query: Query<
        (&VoxelPos, &mut Handle<StandardMaterial>, &mut Visibility),
        With<Overlay>,
    >,
    terrain_handles: Res<TerrainHandles>,
    tile_overlay: Res<TileOverlay>,
    map_geometry: Res<MapGeometry>,
) {
    for (&voxel_pos, mut overlay_material, mut overlay_visibility) in overlay_query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
        let object_interaction = terrain_query.get(terrain_entity).unwrap();

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A discretized 2D vector for visualization purposes.
struct DiscretizedVector {
    /// The direction of the vector.
    direction: DiscretizedDirection,
    /// The magnitude of the vector.
    magnitude: DiscretizedMagnitude,
}

/// A discretized direction, in map coordinate degrees.
///
/// This is used for visualization purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IterableEnum)]
enum DiscretizedDirection {
    /// The direction is approximately 0 degrees.
    Zero,
    /// The direction is approximately 30 degrees.
    Thirty,
    /// The direction is approximately 60 degrees.
    Sixty,
    /// The direction is approximately 90 degrees.
    Ninety,
    /// The direction is approximately 120 degrees.
    OneTwenty,
    /// The direction is approximately 150 degrees.
    OneFifty,
    /// The direction is approximately 180 degrees.
    OneEighty,
    /// The direction is approximately 210 degrees.
    TwoTen,
    /// The direction is approximately 240 degrees.
    TwoForty,
    /// The direction is approximately 270 degrees.
    TwoSeventy,
    /// The direction is approximately 300 degrees.
    ThreeHundred,
    /// The direction is approximately 330 degrees.
    ThreeThirty,
}

impl DiscretizedDirection {
    /// Converts a direction in radians to the nearest discretized direction.
    fn from_radians(radians: f32) -> Self {
        if radians.is_infinite() || radians.is_nan() {
            return DiscretizedDirection::Zero;
        }

        let degrees = radians.to_degrees().rem_euclid(360.);
        assert!(degrees >= 0., "degrees: {degrees}");
        assert!(degrees <= 360., "degrees: {degrees}");

        // Handle the special case of rounding up to 360 degrees
        if degrees > 345.0 {
            return DiscretizedDirection::Zero;
        }

        // PERF: we could use a horrible match statement here, but this is more readable
        let mut nearest = DiscretizedDirection::Zero;
        let mut nearest_distance = 360.;

        for direction in DiscretizedDirection::variants() {
            let distance = (degrees - direction.degrees()).abs();
            if distance < nearest_distance {
                nearest = direction;
                nearest_distance = distance;
            }
        }

        nearest
    }

    /// Returns the angle in degrees of this discretized direction.
    fn degrees(&self) -> f32 {
        match self {
            DiscretizedDirection::Zero => 0.,
            DiscretizedDirection::Thirty => 30.,
            DiscretizedDirection::Sixty => 60.,
            DiscretizedDirection::Ninety => 90.,
            DiscretizedDirection::OneTwenty => 120.,
            DiscretizedDirection::OneFifty => 150.,
            DiscretizedDirection::OneEighty => 180.,
            DiscretizedDirection::TwoTen => 210.,
            DiscretizedDirection::TwoForty => 240.,
            DiscretizedDirection::TwoSeventy => 270.,
            DiscretizedDirection::ThreeHundred => 300.,
            DiscretizedDirection::ThreeThirty => 330.,
        }
    }
}

/// A discretized magnitude of something being visualized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, IterableEnum)]
enum DiscretizedMagnitude {
    /// A magnitude of exactly zero.
    None,
    /// A very weak magnitude.
    VeryWeak,
    /// A weak magnitude.
    Weak,
    /// A moderate magnitude.
    Moderate,
    /// A strong magnitude.
    Strong,
    /// A very strong magnitude.
    VeryStrong,
}

impl DiscretizedMagnitude {
    /// Discretizes a magnitude of water flow into a discretized magnitude.
    fn from_water_flow_volume(volume: Volume) -> DiscretizedMagnitude {
        /// Controls how much water is needed to be considered "very weak", "weak", etc.
        const SCALE_FACTOR: f32 = 1e-2;

        /// Controls how quickly the gap between steps increases.
        const BASE: f32 = 2.0;

        DiscretizedMagnitude::discretize(volume.0, SCALE_FACTOR, BASE)
    }

    /// Discretizes a magnitude.
    ///
    /// The `scale_factor` sets the scale of the magnitude:
    /// its value corresponds to the transition between [`DiscretizedMagnitude::VeryWeak`] and [`DiscretizedMagnitude::Weak`].
    /// The `base` sets the base of the exponent used.
    /// At a base of 0, this is a linear scale. At a base of 10, this is a base-10 logarithmic scale.
    ///
    /// Values of 0.0 or less are considered [`DiscretizedMagnitude::None`].
    fn discretize(magnitude: f32, scale_factor: f32, base: f32) -> Self {
        if magnitude <= 0. {
            DiscretizedMagnitude::None
        } else if magnitude < scale_factor * base.powf(0.) {
            DiscretizedMagnitude::VeryWeak
        } else if magnitude < scale_factor * base.powf(1.) {
            DiscretizedMagnitude::Weak
        } else if magnitude < scale_factor * base.powf(2.) {
            DiscretizedMagnitude::Moderate
        } else if magnitude < scale_factor * base.powf(3.) {
            DiscretizedMagnitude::Strong
        } else {
            DiscretizedMagnitude::VeryStrong
        }
    }

    /// Returns the saturation of this discretized magnitude.
    ///
    /// Weaker magnitudes are less saturated, and stronger magnitudes are more saturated.
    fn saturation(&self) -> f32 {
        match self {
            DiscretizedMagnitude::None => 0.,
            DiscretizedMagnitude::VeryWeak => 0.2,
            DiscretizedMagnitude::Weak => 0.4,
            DiscretizedMagnitude::Moderate => 0.6,
            DiscretizedMagnitude::Strong => 0.8,
            DiscretizedMagnitude::VeryStrong => 1.,
        }
    }
}

/// A marker component for overlay entities.
#[derive(Debug, Clone, Copy, Component)]
struct Overlay;

/// The components used by an entity that is used to visualize spatial information about the world.
#[derive(Bundle)]
struct OverlayBundle {
    /// The marker component for overlay entities.
    overlay: Overlay,
    /// The tile position of the overlay.
    voxel_pos: VoxelPos,
    /// The components needed to render the overlay.
    pbr_bundle: PbrBundle,
}

/// Generates one overlay entity for each valid tile position.
fn spawn_overlay_entities(
    new_terrain_query: Query<&VoxelPos, Added<Id<Terrain>>>,
    mut commands: Commands,
    terrain_handles: Res<TerrainHandles>,
) {
    /// Controls how much larger the overlay is relative to the terrain tiles.
    ///
    /// This must be greater than 0 to avoid z-fighting.
    const EPSILON: f32 = 0.01;

    for &voxel_pos in new_terrain_query.iter() {
        let pbr_bundle = PbrBundle {
            mesh: terrain_handles.topper_mesh.clone_weak(),
            // The material is set in `set_overlay_material`.
            material: Handle::default(),
            // The height of this is fixed in `set_overlay_height`.
            transform: Transform {
                translation: voxel_pos.into_world_pos(),
                scale: Vec3::splat(1. + EPSILON),
                ..Default::default()
            },
            ..Default::default()
        };

        commands.spawn(OverlayBundle {
            overlay: Overlay,
            voxel_pos,
            pbr_bundle,
        });
    }
}

/// Sets the height of each overlay entity based on the height of the terrain and the height of the water.
///
/// Overlays should be positioned just above the water surface when it exists, and just above the terrain when it doesn't.
fn set_overlay_height(
    mut overlay_query: Query<(&VoxelPos, &mut Transform), With<Overlay>>,
    terrain_query: Query<(&WaterDepth, &VoxelPos)>,
    map_geometry: Res<MapGeometry>,
) {
    let topper_thickness = Height::from_world_pos(Height::TOPPER_THICKNESS);

    for (&voxel_pos, mut transform) in overlay_query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
        let (&water_depth, &terrain_pos) = terrain_query.get(terrain_entity).unwrap();
        let terrain_height = terrain_pos.height();

        let desired_height = match water_depth {
            WaterDepth::Dry | WaterDepth::Underground(_) => terrain_height + topper_thickness,
            WaterDepth::Flooded(surface_water_depth) => terrain_height + surface_water_depth,
        };

        transform.translation.y = desired_height.into_world_pos();
    }
}
