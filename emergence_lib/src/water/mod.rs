//! Logic about water movement and behavior.
//!
//! Code for how it should be rendered belongs in the `graphics` module,
//! while code for how it can be used typically belongs in `structures`.

use core::fmt::{Display, Formatter};

use bevy::{prelude::*, utils::HashMap};

use crate::{
    asset_management::manifest::Id,
    items::item_manifest::{Item, ItemManifest},
    simulation::{
        geometry::{Height, MapGeometry, TilePos, Volume},
        SimulationSet,
    },
    structures::structure_manifest::StructureManifest,
};

use self::{
    emitters::{add_water_emitters, produce_water_from_emitters},
    roots::draw_water_from_roots,
    water_dynamics::{evaporation, horizontal_water_movement, precipitation},
};

pub mod emitters;
pub mod roots;
mod water_dynamics;

/// Controls the key parameters of water movement and behavior.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct WaterConfig {
    /// The rate of evaporation per day from each tile.
    evaporation_rate: Height,
    /// The relative rate of evaporation from soil.
    soil_evaporation_ratio: f32,
    /// The rate of precipitation per day on each tile.
    precipitation_rate: Height,
    /// The amount of water that is deposited per day on the tile of each water emitter.
    emission_rate: Volume,
    /// The amount of water that emitters can be covered with before they stop producing.
    emission_pressure: Height,
    /// The number of water items produced for each full tile of water.
    water_items_per_tile: f32,
    /// The amount of water stored in a tile of soil relative to a pure tile of water.
    ///
    /// This value should be less than 1 and must be greater than 0.
    relative_soil_water_capacity: f32,
    /// The rate at which water moves horizontally.
    ///
    /// The units are cubic tiles per day per tile of height difference.
    lateral_flow_rate: f32,
    /// The relative rate at which water moves horizontally through soil.
    soil_lateral_flow_ratio: f32,
}

impl WaterConfig {
    /// The default configuration for in-game water behavior.
    const IN_GAME: Self = Self {
        evaporation_rate: Height(2.0),
        soil_evaporation_ratio: 0.2,
        precipitation_rate: Height(2.0),
        emission_rate: Volume(1e3),
        emission_pressure: Height(1.0),
        water_items_per_tile: 50.0,
        relative_soil_water_capacity: 0.3,
        lateral_flow_rate: 1e3,
        soil_lateral_flow_ratio: 0.2,
    };

    /// A configuration that disables all water behavior.
    #[allow(dead_code)]
    const NULL: Self = Self {
        evaporation_rate: Height(0.0),
        soil_evaporation_ratio: 0.0,
        precipitation_rate: Height(0.0),
        emission_rate: Volume(0.0),
        emission_pressure: Height(0.0),
        water_items_per_tile: 0.0,
        relative_soil_water_capacity: 0.5,
        lateral_flow_rate: 0.0,
        soil_lateral_flow_ratio: 0.0,
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
enum WaterSet {
    /// Systems that increase or decrease the amount of water in the water table.
    VerticalWaterMovement,
    /// Systems that move water horizontally.
    HorizontalWaterMovement,
    /// Systems that synchronize the state of the world.
    Synchronization,
}

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTable>()
            .insert_resource(WaterConfig::IN_GAME);

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
                .add_systems(
                    (
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
                        .before(WaterSet::HorizontalWaterMovement),
                )
                .add_system(horizontal_water_movement.in_set(WaterSet::HorizontalWaterMovement))
                .add_systems(
                    (add_water_emitters, update_water_depth).in_set(WaterSet::Synchronization),
                );
        });
    }
}

/// The amount and height of water across the map.
///
/// This can be underground, at ground level, or above ground.
/// If it is above ground, it will pool on top of the tile it is on.
#[derive(Resource, Default, PartialEq, Clone, Debug)]
pub struct WaterTable {
    /// The volume of water at each tile.
    volume: HashMap<TilePos, Volume>,
    /// The height of the water table at each tile relative to the soil surface.
    ///
    /// This is updated in [`update_water_depth`], and cached for both performance and plumbing reasons.
    water_depth: HashMap<TilePos, WaterDepth>,
}

impl WaterTable {
    /// Gets the total volume of water on the given tile.
    pub(crate) fn get_volume(&self, tile_pos: TilePos) -> Volume {
        self.volume.get(&tile_pos).copied().unwrap_or_default()
    }

    /// Gets the height of the water table at the given tile.
    pub(crate) fn get_height(&self, tile_pos: TilePos, map_geometry: &MapGeometry) -> Height {
        let soil_height = map_geometry.get_height(tile_pos).unwrap();

        match self.water_depth(tile_pos) {
            WaterDepth::Dry => Height::ZERO,
            WaterDepth::Underground(depth) => soil_height - depth,
            WaterDepth::Flooded(depth) => soil_height + depth,
        }
    }

    /// Computes the height of water that is above the soil at `tile_pos`.
    pub(crate) fn surface_water_depth(&self, tile_pos: TilePos) -> Height {
        let depth_to_water_table = self.water_depth(tile_pos);

        match depth_to_water_table {
            WaterDepth::Dry => Height::ZERO,
            WaterDepth::Underground(..) => Height::ZERO,
            WaterDepth::Flooded(depth) => depth,
        }
    }

    /// Get the depth of the water table at the given tile relative to the soil surface.
    pub(crate) fn water_depth(&self, tile_pos: TilePos) -> WaterDepth {
        self.water_depth.get(&tile_pos).copied().unwrap_or_default()
    }

    /// Sets the total volume of water at the given tile.
    pub(crate) fn set_volume(&mut self, tile_pos: TilePos, volume: Volume) {
        self.volume.insert(tile_pos, volume);
    }

    /// Adds the given amount of water to the water table at the given tile.
    pub(crate) fn add(&mut self, tile_pos: TilePos, amount: Volume) {
        let height = self.get_volume(tile_pos);
        let new_height = height + amount;
        self.set_volume(tile_pos, new_height);
    }

    /// Subtracts the given amount of water from the water table at the given tile.
    ///
    /// This will never return a height below zero.
    ///
    /// Returns the amount of water that was actually subtracted.
    pub(crate) fn remove(&mut self, tile_pos: TilePos, amount: Volume) -> Volume {
        let height = self.get_volume(tile_pos);
        // We cannot take more water than there is.
        let water_drawn = amount.min(height);
        let new_height = height - water_drawn;
        self.set_volume(tile_pos, new_height);
        water_drawn
    }

    /// Computes the total volume of water in the water table.
    pub(crate) fn total_water(&self) -> Volume {
        self.volume
            .values()
            .copied()
            .reduce(|a, b| a + b)
            .unwrap_or_default()
    }

    /// Computes the average height of the water table.
    pub(crate) fn average_height(&self, map_geometry: &MapGeometry) -> Height {
        let total_water = self.total_water();
        let total_area = map_geometry.valid_tile_positions().count() as f32;
        (total_water / total_area).into_height()
    }
}

impl Id<Item> {
    /// The identifier for the water item.
    // This can't be a const because Rust hates for loops in const functions T_T
    pub(crate) fn water() -> Self {
        Self::from_name("water".to_string())
    }
}

/// The depth of the water table at a given tile relative to the soil surface.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum WaterDepth {
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
        relative_soil_water_capacity: f32,
    ) -> WaterDepth {
        if water_volume == Volume::ZERO {
            return WaterDepth::Dry;
        }

        let max_volume_stored_by_soil =
            Volume::from_height(soil_height * relative_soil_water_capacity);

        if max_volume_stored_by_soil >= water_volume {
            // If the soil water capacity is low, then we will need more height to store the same volume of water.
            let height_of_water_stored_by_soil =
                water_volume.into_height() / relative_soil_water_capacity;

            let depth = soil_height - height_of_water_stored_by_soil;
            WaterDepth::Underground(depth)
        } else {
            let above_surface_volume = water_volume - max_volume_stored_by_soil;
            WaterDepth::Flooded(above_surface_volume.into_height())
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

/// Updates the depth of water at each tile based on the volume of water and soil properties.
fn update_water_depth(
    mut water_table: ResMut<WaterTable>,
    water_config: Res<WaterConfig>,
    map_geometry: Res<MapGeometry>,
) {
    water_table.water_depth.clear();

    for tile_pos in map_geometry.valid_tile_positions() {
        let soil_height = map_geometry.get_height(tile_pos).unwrap();
        let water_volume = water_table.get_volume(tile_pos);

        let water_depth = WaterDepth::compute(
            water_volume,
            soil_height,
            // TODO: vary this based on soil type
            water_config.relative_soil_water_capacity,
        );

        water_table.water_depth.insert(tile_pos, water_depth);
    }
}
