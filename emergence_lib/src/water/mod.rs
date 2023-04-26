//! Logic about water movement and behavior.
//!
//! Code for how it should be rendered belongs in the `graphics` module,
//! while code for how it can be used typically belongs in `structures`.

use bevy::prelude::*;

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
#[derive(Resource)]
pub struct WaterTable {
    /// The single global height of the water table.
    height: Height,
}

impl Default for WaterTable {
    fn default() -> Self {
        Self { height: Height(2.) }
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

        if water_table.height > tile_height {
            map_geometry.add_surface_water(tile_pos, water_table.height - tile_height);
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
    const EVAPORATION_PER_DAY: Height = Height(0.5);
    let evaporation_per_second = EVAPORATION_PER_DAY.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let evaporation_rate =
        evaporation_per_second * elapsed_time * current_weather.get().evaporation_rate();

    let n_exposed_tiles = map_geometry
        .valid_tile_positions()
        .filter(|tile_pos| map_geometry.get_surface_water_height(*tile_pos).is_some())
        .count();
    let total_tiles = map_geometry.valid_tile_positions().count();

    let total_evaporation = evaporation_rate * n_exposed_tiles as f32 / total_tiles as f32;
    water_table.height -= Height(total_evaporation);
}

/// Adds water to the water table via rainfall.
fn precipitation(
    mut water_table: ResMut<WaterTable>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
) {
    /// The amount of water that is deposited per day on each tile.
    const PRECIPITATION_PER_DAY: Height = Height(0.5);
    let precipitation_per_second = PRECIPITATION_PER_DAY.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let precipitation_rate =
        precipitation_per_second * elapsed_time * current_weather.get().precipitation_rate();
    water_table.height += Height(precipitation_rate);
}
