//! Logic about water movement and behavior.
//!
//! Code for how it should be rendered belongs in the `graphics` module,
//! while code for how it can be used typically belongs in `structures`.

use core::fmt::{Display, Formatter};

use bevy::{prelude::*, utils::HashMap};

use crate::simulation::{
    geometry::{Height, MapGeometry, TilePos},
    time::InGameTime,
    weather::CurrentWeather,
    SimulationSet,
};

/// A plugin that handles water movement and behavior.
pub(super) struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTable>().add_systems(
            (
                evaporation,
                precipitation,
                horizontal_water_movement,
                update_surface_water_map_geometry,
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// The height of the water.
///
/// This can be underground, at ground level, or above ground.
/// If it is above ground, it will pool on top of the tile it is on.
#[derive(Resource, Default)]
pub(crate) struct WaterTable {
    /// The height of the water table at each tile.
    height: HashMap<TilePos, Height>,
}

impl WaterTable {
    /// Gets the height of the water table at the given tile.
    pub(crate) fn get(&self, tile_pos: TilePos) -> Height {
        self.height.get(&tile_pos).copied().unwrap_or_default()
    }

    /// Get the depth to the water table at the given tile.
    ///
    /// If there is surface water, this will be zero.
    pub(crate) fn depth_to_water_table(
        &self,
        tile_pos: TilePos,
        map_geometry: &MapGeometry,
    ) -> DepthToWaterTable {
        let tile_height = map_geometry.get_height(tile_pos).unwrap();
        let water_height = self.get(tile_pos);
        if water_height == Height::ZERO {
            DepthToWaterTable::Dry
        } else if water_height >= tile_height {
            DepthToWaterTable::Flooded
        } else {
            DepthToWaterTable::Depth(tile_height - water_height)
        }
    }

    /// Sets the height of the water table at the given tile.
    pub(crate) fn set(&mut self, tile_pos: TilePos, height: Height) {
        self.height.insert(tile_pos, height);
    }

    /// Adds the given amount of water to the water table at the given tile.
    pub(crate) fn add(&mut self, tile_pos: TilePos, amount: Height) {
        let height = self.get(tile_pos);
        let new_height = height + amount;
        self.set(tile_pos, new_height);
    }

    /// Subtracts the given amount of water from the water table at the given tile.
    ///
    /// This will not go below zero.
    pub(crate) fn subtract(&mut self, tile_pos: TilePos, amount: Height) {
        let height = self.get(tile_pos);
        let new_height = height - amount;
        self.set(tile_pos, new_height.max(Height::ZERO));
    }
}

/// The depth to the water table at a given tile.
#[derive(Debug)]
pub(crate) enum DepthToWaterTable {
    /// The water table is above the surface.
    Flooded,
    /// The water table is completely empty.
    Dry,
    /// The water table is at the given depth, measured from the soil surface.
    Depth(Height),
}

impl Display for DepthToWaterTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DepthToWaterTable::Flooded => write!(f, "Flooded"),
            DepthToWaterTable::Dry => write!(f, "Dry"),
            DepthToWaterTable::Depth(depth) => write!(f, "{depth} from surface"),
        }
    }
}

/// Computes how much water is on the surface of each tile.
fn update_surface_water_map_geometry(
    mut map_geometry: ResMut<MapGeometry>,
    water_table: Res<WaterTable>,
) {
    // Collect out to avoid borrow checker pain
    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        let tile_height = map_geometry.get_height(tile_pos).unwrap();
        let water_height = water_table.get(tile_pos);

        if water_height > tile_height {
            map_geometry.add_surface_water(tile_pos, water_height - tile_height);
        } else {
            map_geometry.remove_surface_water(tile_pos);
        }
    }
}

/// Evaporates water from surface water.
fn evaporation(
    mut water_table: ResMut<WaterTable>,
    map_geometry: Res<MapGeometry>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
) {
    /// The amount of water that evaporates per day from each surface water tile.
    const EVAPORATION_PER_DAY: Height = Height(1.5);
    /// The relative rate of evaporation from soil relative to surface water.
    const SOIL_EVAPORATION_RATE: f32 = 0.5;

    let evaporation_per_second = EVAPORATION_PER_DAY.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let evaporation_rate =
        evaporation_per_second * elapsed_time * current_weather.get().evaporation_rate();

    for tile in map_geometry.valid_tile_positions() {
        // Surface water evaporation
        let total_evaporated = match map_geometry.get_surface_water_height(tile) {
            Some(_) => Height(evaporation_rate),
            None => Height(evaporation_rate * SOIL_EVAPORATION_RATE),
        };

        water_table.subtract(tile, total_evaporated);
    }
}

/// Adds water to the water table via rainfall.
fn precipitation(
    mut water_table: ResMut<WaterTable>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
    map_geometry: Res<MapGeometry>,
) {
    /// The amount of water that is deposited per day on each tile.
    const PRECIPITATION_PER_DAY: Height = Height(1.0);
    let precipitation_per_second = PRECIPITATION_PER_DAY.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let precipitation_rate = Height(
        precipitation_per_second * elapsed_time * current_weather.get().precipitation_rate(),
    );

    for tile_pos in map_geometry.valid_tile_positions() {
        water_table.add(tile_pos, precipitation_rate);
    }
}

/// Moves water from one tile to another, according to the relative height of the water table.
fn horizontal_water_movement(
    mut water_table: ResMut<WaterTable>,
    map_geometry: Res<MapGeometry>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
) {
    /// The rate of water transfer between adjacent tiles.
    ///
    /// The units are cubic tiles per day per tile of height difference.
    const WATER_FLOW_RATE: f32 = 0.5;
    let water_flow_coefficient =
        WATER_FLOW_RATE / in_game_time.seconds_per_day() * fixed_time.period.as_secs_f32();

    // We must use a working copy of the water table to avoid effects due to the order of evaluation.
    let mut delta_water_flow = WaterTable::default();
    for tile_pos in map_geometry.valid_tile_positions() {
        let height = water_table.get(tile_pos);
        let neighbors = tile_pos.all_neighbors(&map_geometry);
        for neighbor in neighbors {
            let neighbor_height = water_table.get(neighbor);
            // FIXME: this is non-conservative; water can be moved even from tiles that end up being overdrawn
            // If the water is higher than the neighbor, move water from the tile to the neighbor
            // at a rate proportional to the height difference.
            // If the water is lower than the neighbor, the flow direction is reversed.
            // The rate is halved as we do the same computation in both directions.
            let delta_water_height = height - neighbor_height;
            let water_transfer = delta_water_height * water_flow_coefficient / 2.;
            delta_water_flow.subtract(tile_pos, water_transfer);
            delta_water_flow.add(neighbor, water_transfer);
        }
    }

    // Apply the changes
    for tile_pos in map_geometry.valid_tile_positions() {
        water_table.add(tile_pos, delta_water_flow.get(tile_pos));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn water_table_arithmetic() {
        use super::*;
        let mut water_table = WaterTable::default();
        let tile_pos = TilePos::new(0, 0);
        water_table.set(tile_pos, Height(1.0));
        assert_eq!(water_table.get(tile_pos), Height(1.0));

        water_table.add(tile_pos, Height(1.0));
        assert_eq!(water_table.get(tile_pos), Height(2.0));

        water_table.subtract(tile_pos, Height(1.0));
        assert_eq!(water_table.get(tile_pos), Height(1.0));

        water_table.subtract(tile_pos, Height(1.0));
        assert_eq!(water_table.get(tile_pos), Height(0.0));

        water_table.subtract(tile_pos, Height(1.0));
        assert_eq!(water_table.get(tile_pos), Height(0.0));
    }
}
