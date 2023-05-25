//! Logic about water movement and behavior.
//!
//! Code for how it should be rendered belongs in the `graphics` module,
//! while code for how it can be used typically belongs in `structures`.

use core::fmt::{Display, Formatter};
use core::ops::{Div, Mul};
use std::f32::consts::PI;

use bevy::prelude::*;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use serde::{Deserialize, Serialize};

use crate::simulation::time::Days;
use crate::{
    asset_management::manifest::Id,
    items::item_manifest::{Item, ItemManifest},
    simulation::{
        geometry::{Height, MapGeometry, Volume},
        SimulationSet,
    },
    structures::structure_manifest::StructureManifest,
};

use self::ocean::{tides, Ocean, TideSettings};
use self::water_dynamics::{SoilWaterEvaporationRate, SoilWaterFlowRate};
use self::{
    emitters::{add_water_emitters, produce_water_from_emitters},
    roots::draw_water_from_roots,
    water_dynamics::{evaporation, horizontal_water_movement, precipitation},
};

pub mod emitters;
pub mod ocean;
pub mod roots;
pub mod water_dynamics;

/// Controls the key parameters of water movement and behavior.
///
/// Note that soil properties are stored seperately for each soil type in [`TerrainData`](crate::terrain::terrain_manifest::TerrainData).
#[derive(Resource, Debug, Clone, Copy)]
pub struct WaterConfig {
    /// The rate of evaporation per day from each tile.
    pub evaporation_rate: Height,
    /// The rate of precipitation per day on each tile.
    pub precipitation_rate: Height,
    /// The amount of water that is deposited per day on the tile of each water emitter.
    pub emission_rate: Volume,
    /// The amount of water that emitters can be covered with before they stop producing.
    pub emission_pressure: Height,
    /// The number of water items produced for each full tile of water.
    pub water_items_per_tile: f32,
    /// The rate at which water moves horizontally.
    ///
    /// The units are cubic tiles per day per tile of height difference.
    ///
    /// # Warning
    ///
    /// If this value becomes too large, the simulation may become unstable, with water alternating between fully flooded and fully dry tiles.
    pub lateral_flow_rate: f32,
    /// Are oceans enabled?
    pub enable_oceans: bool,
    /// Controls the behavior of the tides.
    pub tide_settings: TideSettings,
}

impl WaterConfig {
    /// The default configuration for in-game water behavior.
    pub const IN_GAME: Self = Self {
        evaporation_rate: Height(2.0),
        precipitation_rate: Height(2.0),
        emission_rate: Volume(1e4),
        emission_pressure: Height(5.0),
        water_items_per_tile: 50.0,
        lateral_flow_rate: 1e4,
        enable_oceans: true,
        tide_settings: TideSettings {
            amplitude: Height(1.0),
            period: Days(1.5),
            minimum: Height(0.0),
        },
    };

    /// A configuration that disables all water behavior.
    pub const NULL: Self = Self {
        evaporation_rate: Height(0.0),
        precipitation_rate: Height(0.0),
        emission_rate: Volume(0.0),
        emission_pressure: Height(0.0),
        water_items_per_tile: 0.0,
        lateral_flow_rate: 0.0,
        enable_oceans: false,
        tide_settings: TideSettings {
            amplitude: Height(0.0),
            period: Days(1.0),
            minimum: Height(0.0),
        },
    };

    /// Converts a number of items of water to a [`Volume`] of water.
    pub(crate) fn items_to_tiles(&self, items: u32) -> Volume {
        Volume(items as f32 / self.water_items_per_tile)
    }

    /// Converts a [`Volume`] of water to an equivalent number of items of water.
    pub(crate) fn tiles_to_items(&self, height: Volume) -> u32 {
        (height.0 * self.water_items_per_tile) as u32
    }
}

/// A plugin that handles water movement and behavior.
pub(super) struct WaterPlugin;

/// System set for water movement and behavior.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WaterSet {
    /// Systems that increase or decrease the amount of water in the water table.
    VerticalWaterMovement,
    /// Systems that move water horizontally.
    HorizontalWaterMovement,
    /// Systems that synchronize the state of the world.
    Synchronization,
}

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaterConfig::IN_GAME)
            .init_resource::<Ocean>();

        app.edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
            schedule
                .configure_sets(
                    (
                        WaterSet::VerticalWaterMovement,
                        WaterSet::HorizontalWaterMovement,
                        WaterSet::Synchronization,
                    )
                        .in_set(SimulationSet)
                        .chain(),
                )
                .add_system(
                    cache_water_volume
                        .before(WaterSet::VerticalWaterMovement)
                        // This needs to respect pausing
                        .in_set(SimulationSet),
                )
                .add_systems(
                    (
                        tides,
                        produce_water_from_emitters,
                        precipitation,
                        // This system pulls in a ton of dependencies, so it's best to fail silently when they don't exist
                        // to allow for integration testing of water behavior.
                        draw_water_from_roots
                            .run_if(resource_exists::<StructureManifest>())
                            .run_if(resource_exists::<ItemManifest>()),
                        evaporation,
                    )
                        .chain()
                        .in_set(WaterSet::VerticalWaterMovement),
                )
                .add_system(
                    // It is important that the computed height of the water is accurate before we start moving it around.
                    update_water_depth
                        .after(WaterSet::VerticalWaterMovement)
                        .before(WaterSet::HorizontalWaterMovement)
                        .in_set(SimulationSet),
                )
                .add_system(horizontal_water_movement.in_set(WaterSet::HorizontalWaterMovement))
                .add_systems(
                    (add_water_emitters, update_water_depth).in_set(WaterSet::Synchronization),
                );
        });
    }
}

impl Id<Item> {
    /// The identifier for the water item.
    // This can't be a const because Rust hates for loops in const functions T_T
    pub(crate) fn water() -> Self {
        Self::from_name("water".to_string())
    }
}

/// The components needed to track the water table.
///
/// These are stored on terrain tile entities.
/// To fully compute basic water dynamics, you also need the [`TilePos`](crate::simulation::geometry::TilePos) and [`Height`] components.
#[derive(Bundle, Debug, Default)]
pub struct WaterBundle {
    /// The volume of water stored at this tile.
    pub water_volume: WaterVolume,
    /// The volume of water stored at this tile the previous tick.
    pub previous_water_volume: PreviousWaterVolume,
    /// The rate and direction of water flow at this tile.
    pub flow_velocity: FlowVelocity,
    /// The depth of water at this tile.
    pub water_depth: WaterDepth,
    /// The amount of water that can be stored at this tile.
    pub soil_water_capacity: SoilWaterCapacity,
    /// The relative rate at which water evaporates from this tile.
    pub soil_water_evaporation_rate: SoilWaterEvaporationRate,
    /// The rate at which soil water flows through this tile.
    pub soil_water_flow_rate: SoilWaterFlowRate,
}

/// The depth of the water table at a given tile relative to the soil surface.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum WaterDepth {
    /// The water table is completely empty.
    #[default]
    Dry,
    /// The water table beneath the soil.
    ///
    /// The depth is measured down from the soil surface.
    Underground(Height),
    /// The water table is above the surface.
    ///
    /// The height is the height above the soil surface.
    Flooded(Height),
}

impl WaterDepth {
    /// Computes the [`WaterDepth`] for a single tile.
    #[inline]
    #[must_use]
    fn compute(
        water_volume: Volume,
        soil_height: Height,
        relative_soil_water_capacity: SoilWaterCapacity,
    ) -> WaterDepth {
        if water_volume == Volume::ZERO {
            return WaterDepth::Dry;
        }

        let max_volume_stored_by_soil =
            Volume::from_height(soil_height * relative_soil_water_capacity.0);

        if max_volume_stored_by_soil >= water_volume {
            // If the soil water capacity is low, then we will need more height to store the same volume of water.
            let height_of_water_stored_by_soil =
                water_volume.into_height() / relative_soil_water_capacity.0;

            let depth = soil_height - height_of_water_stored_by_soil;
            WaterDepth::Underground(depth)
        } else {
            let above_surface_volume = water_volume - max_volume_stored_by_soil;
            WaterDepth::Flooded(above_surface_volume.into_height())
        }
    }

    /// Computes the height of the surface water, or the terrain height if there is no water.
    pub(crate) fn surface_height(&self, terrain_height: Height) -> Height {
        match self {
            WaterDepth::Dry => terrain_height,
            WaterDepth::Underground(..) => terrain_height,
            WaterDepth::Flooded(depth) => terrain_height + *depth,
        }
    }

    /// Computes the depth of the surface water above the soil surface.
    ///
    /// Returns [`Height::ZERO`] if there is no surface water.
    pub(crate) fn surface_water_depth(&self) -> Height {
        match self {
            WaterDepth::Dry => Height::ZERO,
            WaterDepth::Underground(..) => Height::ZERO,
            WaterDepth::Flooded(depth) => *depth,
        }
    }

    /// Computes the absolute height of the water table.
    pub(crate) fn water_table_height(&self, terrain_height: Height) -> Height {
        match self {
            WaterDepth::Dry => Height::ZERO,
            WaterDepth::Underground(depth) => terrain_height - *depth,
            WaterDepth::Flooded(depth) => terrain_height + *depth,
        }
    }
}

impl Display for WaterDepth {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            WaterDepth::Dry => write!(f, "Dry"),
            WaterDepth::Underground(depth) => write!(f, "{depth} below surface"),
            WaterDepth::Flooded(depth) => write!(f, "{depth} above surface"),
        }
    }
}

/// The relative volume of water that can be stored in the soil.
///
/// This is relative to water above the soil, which has a value of 1.0.
/// As a result, this value is always between 0.0 and 1.0.
#[derive(Component, Clone, Copy, Debug, Add, Sub, PartialEq, Serialize, Deserialize)]
pub struct SoilWaterCapacity(pub f32);

impl Default for SoilWaterCapacity {
    fn default() -> Self {
        Self(0.5)
    }
}

/// The amount of water stored on this terrain tile.
#[derive(
    Component, Default, Clone, Copy, Debug, Add, Sub, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct WaterVolume(Volume);

impl Mul<f32> for WaterVolume {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for WaterVolume {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl WaterVolume {
    /// Represents no water.
    pub const ZERO: Self = Self(Volume::ZERO);

    /// Creates a new [`WaterVolume`] with the given amount of water.
    ///
    /// # Panics
    ///
    /// Panics if the given volume is negative.
    #[must_use]
    pub fn new(volume: Volume) -> Self {
        assert!(volume >= Volume::ZERO);

        Self(volume)
    }

    /// Adds the given amount of water to the tile.
    pub(crate) fn add(&mut self, volume: Volume) {
        self.0 += volume;
    }

    /// Subtracts the given amount of water from the tile.
    ///
    /// Returns the amount of water that was actually removed.
    pub(crate) fn remove(&mut self, volume: Volume) -> Volume {
        if self.0 < volume {
            let removed = self.0;
            self.0 = Volume::ZERO;
            removed
        } else {
            self.0 -= volume;
            volume
        }
    }

    /// Gets the amount of water stored on this tile.
    pub(crate) fn volume(&self) -> Volume {
        self.0
    }

    /// Returns the absolute difference in water volume between this tile and the other tile.
    #[cfg(test)]
    pub(crate) fn abs_diff(&self, other: Self) -> Volume {
        self.0.abs_diff(other.0)
    }
}

/// The water volume at this tile on the previous tick.
#[derive(Component, Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct PreviousWaterVolume(pub(crate) WaterVolume);

/// Updates the depth of water at each tile based on the volume of water and soil properties.
pub fn update_water_depth(
    mut query: Query<(&Height, &WaterVolume, &SoilWaterCapacity, &mut WaterDepth)>,
) {
    // Critically, the depth of tiles *outside* of the map is not updated here.
    // Instead, they are implicitly treated as the ocean depth.
    // As a result, the ocean acts as both an infinite source and sink for water.
    for (&soil_height, water_volume, &relative_soil_water_capacity, mut water_depth) in
        query.iter_mut()
    {
        *water_depth =
            WaterDepth::compute(water_volume.0, soil_height, relative_soil_water_capacity);
    }
}

/// The rate and direction of lateral water flow.
#[derive(Component, Debug, Default, PartialEq, Clone, Add, AddAssign, Sub, SubAssign)]
pub struct FlowVelocity {
    /// The x component (in world coordinates) of the flow velocity.
    x: Volume,
    /// The z component (in world coordinates) of the flow velocity.
    z: Volume,
}

impl FlowVelocity {
    /// The 0 vector of flow velocity.
    pub(crate) const ZERO: Self = Self {
        x: Volume::ZERO,
        z: Volume::ZERO,
    };

    /// The magnitude of the flow velocity.
    #[inline]
    #[must_use]
    pub(crate) fn magnitude(&self) -> Volume {
        Volume((self.x.0.powi(2) + self.z.0.powi(2)).sqrt())
    }

    /// The direction of the flow velocity in radians.
    #[inline]
    #[must_use]
    pub(crate) fn direction(&self) -> f32 {
        self.z.0.atan2(self.x.0)
    }

    /// Converts a [`hexx::Direction`] and magnitude into a [`FlowVelocity`].
    fn from_hex_direction(
        direction: hexx::Direction,
        magnitude: Volume,
        map_geometry: &MapGeometry,
    ) -> Self {
        // Empirically this seems to be the correct angle.
        let angle = direction.angle(&map_geometry.layout.orientation) + PI;
        let x = magnitude * angle.cos();
        let z = magnitude * angle.sin();

        Self { x, z }
    }
}

impl Mul<f32> for FlowVelocity {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<FlowVelocity> for f32 {
    type Output = FlowVelocity;

    #[inline]
    fn mul(self, rhs: FlowVelocity) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for FlowVelocity {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            z: self.z / rhs,
        }
    }
}

/// Records the previous water volume for each tile at the start of the tick.
///
/// This is later used to compute the flow velocity and rate of water flux.
fn cache_water_volume(mut water_query: Query<(&WaterVolume, &mut PreviousWaterVolume)>) {
    for (&water_volume, mut previous_water_volume) in water_query.iter_mut() {
        previous_water_volume.0 = water_volume;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_depth_returns_dry_when_volume_is_zero() {
        let water_depth = WaterDepth::compute(Volume::ZERO, Height::ZERO, SoilWaterCapacity(0.5));

        assert_eq!(water_depth, WaterDepth::Dry);

        let water_depth = WaterDepth::compute(Volume::ZERO, Height(1.0), SoilWaterCapacity(0.5));
        assert_eq!(water_depth, WaterDepth::Dry);
    }

    #[test]
    fn water_depth_returns_underground_when_volume_is_less_than_soil_capacity() {
        let water_depth: WaterDepth = WaterDepth::compute(
            Volume::from_height(Height(0.1)),
            Height(1.0),
            SoilWaterCapacity(0.5),
        );
        assert_eq!(water_depth, WaterDepth::Underground(Height(0.8)));

        let water_depth = WaterDepth::compute(
            Volume::from_height(Height(0.5)),
            Height(1.0),
            SoilWaterCapacity(0.5),
        );
        assert_eq!(water_depth, WaterDepth::Underground(Height(0.0)));
    }

    #[test]
    fn water_depth_returns_flooded_when_volume_is_greater_than_soil_capacity() {
        let water_depth: WaterDepth = WaterDepth::compute(
            Volume::from_height(Height(1.0)),
            Height(1.0),
            SoilWaterCapacity(0.5),
        );
        assert_eq!(water_depth, WaterDepth::Flooded(Height(0.5)));
    }
}
