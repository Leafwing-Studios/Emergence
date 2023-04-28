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

use self::{emitters::produce_water_from_emitters, roots::draw_water_from_roots};

mod emitters;
pub mod roots;

/// Controls the key parameters of water movement and behavior.
#[derive(Resource, Debug, Default)]
struct WaterConfig {
    /// The rate of evaporation per day from each tile.
    evaporation_rate: Height,
    /// The relative rate of evaporation from soil.
    soil_evaporation_ratio: f32,
    /// The rate of precipitation per day on each tile.
    precipitation_rate: Height,
    /// The amount of water that is deposited per day on the tile of each water emitter.
    emission_rate: Height,
    /// The amount of water that is drawn per day from the tile of each structure with roots.
    root_draw_rate: Height,
    /// The rate at which water moves horizontally.
    ///
    /// The units are cubic tiles per day per tile of height difference.
    lateral_water_flow_rate: f32,
    /// The relative rate at which water moves horizontally through soil.
    soil_lateral_water_flow_ratio: f32,
}

/// A plugin that handles water movement and behavior.
pub(super) struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTable>()
            .insert_resource(WaterConfig {
                evaporation_rate: Height(0.0),
                soil_evaporation_ratio: 0.0,
                precipitation_rate: Height(0.0),
                emission_rate: Height(100.0),
                root_draw_rate: Height(0.0),
                lateral_water_flow_rate: 10.0,
                soil_lateral_water_flow_ratio: 0.5,
            })
            .add_systems(
                (
                    evaporation,
                    precipitation,
                    horizontal_water_movement,
                    produce_water_from_emitters,
                    draw_water_from_roots,
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

    /// Computes the total amount of water in the water table.
    pub(crate) fn total_water(&self) -> Height {
        self.height
            .values()
            .copied()
            .reduce(|a, b| a + b)
            .unwrap_or_default()
    }

    /// Computes the average depth of the water table.
    pub(crate) fn average_depth(&self, map_geometry: &MapGeometry) -> Height {
        let total_water = self.total_water();
        let total_area = map_geometry.valid_tile_positions().count() as f32;
        total_water / total_area
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
    water_config: Res<WaterConfig>,
    map_geometry: Res<MapGeometry>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
) {
    let evaporation_per_second = water_config.evaporation_rate.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let evaporation_rate =
        evaporation_per_second * elapsed_time * current_weather.get().evaporation_rate();

    for tile in map_geometry.valid_tile_positions() {
        // Surface water evaporation
        let total_evaporated = match map_geometry.get_surface_water_height(tile) {
            Some(_) => Height(evaporation_rate),
            None => Height(evaporation_rate * water_config.soil_evaporation_ratio),
        };

        water_table.subtract(tile, total_evaporated);
    }
}

/// Adds water to the water table via rainfall.
fn precipitation(
    mut water_table: ResMut<WaterTable>,
    water_config: Res<WaterConfig>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
    map_geometry: Res<MapGeometry>,
) {
    let precipitation_per_second =
        water_config.precipitation_rate.0 / in_game_time.seconds_per_day();
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
    water_config: Res<WaterConfig>,
    map_geometry: Res<MapGeometry>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
) {
    let base_water_transfer_amount = water_config.lateral_water_flow_rate
        / in_game_time.seconds_per_day()
        * fixed_time.period.as_secs_f32();

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

            // Water flows more easily between tiles that are both flooded.
            let medium_coefficient = match (
                map_geometry.get_surface_water_height(tile_pos).is_some(),
                map_geometry.get_surface_water_height(neighbor).is_some(),
            ) {
                (true, true) => 1.,
                (false, false) => water_config.soil_lateral_water_flow_ratio,
                _ => (1. + water_config.soil_lateral_water_flow_ratio) / 2.,
            };

            let water_transfer =
                delta_water_height * medium_coefficient * base_water_transfer_amount / 2.;
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
    use emergence_macros::IterableEnum;
    use rand::Rng;

    use crate as emergence_lib;
    use crate::enum_iter::IterableEnum;
    use crate::simulation::time::advance_in_game_time;

    use super::*;

    fn water_testing_app(water_table: WaterTable, map_geometry: MapGeometry) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugin(WaterPlugin)
            .add_system(
                advance_in_game_time
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );

        for &tile_pos in water_table.height.keys() {
            assert!(
                map_geometry.is_valid(tile_pos),
                "Invalid tile position {} found in water table.",
                tile_pos
            );
        }

        app.insert_resource(water_table);
        app.insert_resource(map_geometry);

        app
    }

    /// Controls the initial water level of the map.
    #[derive(IterableEnum)]
    enum WaterTableScenario {
        /// No water.
        Dry,
        /// Half a tile of water.
        DepthHalf,
        /// One tile of water.
        DepthOne,
    }

    impl WaterTableScenario {
        fn starting_water_level(&self) -> Height {
            match self {
                WaterTableScenario::Dry => Height(0.),
                WaterTableScenario::DepthHalf => Height(0.5),
                WaterTableScenario::DepthOne => Height(1.),
            }
        }

        fn water_table(&self, map_geometry: &MapGeometry) -> WaterTable {
            let mut water_table = WaterTable::default();
            for tile_pos in map_geometry.valid_tile_positions() {
                water_table.set(tile_pos, self.starting_water_level());
            }

            water_table
        }
    }

    /// The size of the test map.
    #[derive(IterableEnum)]
    enum MapSizes {
        /// Radius 0 map.
        OneTile,
        /// Radius 3 map.
        Tiny,
        /// Radius 10 map.
        Modest,
    }

    impl MapSizes {
        fn map_geometry(&self) -> MapGeometry {
            match self {
                MapSizes::OneTile => MapGeometry::new(0),
                MapSizes::Tiny => MapGeometry::new(3),
                MapSizes::Modest => MapGeometry::new(10),
            }
        }
    }

    /// The shape of the test map.
    #[derive(IterableEnum)]
    enum MapShape {
        /// A flat map with no variation in height at height 0.
        Bedrock,
        /// A flat map with no variation in height at height 1.
        Flat,
        /// A map that slopes.
        Sloped,
        /// A map with random bumps.
        Bumpy,
    }

    impl MapShape {
        fn set_heights(&self, mut map_geometry: MapGeometry) -> MapGeometry {
            for tile_pos in map_geometry
                .valid_tile_positions()
                .collect::<Vec<TilePos>>()
            {
                let height = match self {
                    MapShape::Bedrock => Height(0.),
                    MapShape::Flat => Height(1.),
                    MapShape::Sloped => Height(tile_pos.x as f32),
                    MapShape::Bumpy => {
                        let rng = &mut rand::thread_rng();
                        Height(rng.gen())
                    }
                };

                map_geometry.update_height(tile_pos, height);
            }

            map_geometry
        }
    }

    #[test]
    fn water_table_arithmetic() {
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

    #[test]
    fn evaporation_decreases_water_levels() {
        for map_size in MapSizes::variants() {
            for map_shape in MapShape::variants() {
                for scenario in WaterTableScenario::variants() {
                    let map_geometry = map_shape.set_heights(map_size.map_geometry());
                    let water_table = scenario.water_table(&map_geometry);
                    let mut app = water_testing_app(water_table, map_geometry);
                    app.update();

                    let water_table = app.world.resource::<WaterTable>();

                    for &tile_pos in water_table.height.keys() {
                        if scenario.starting_water_level() > Height::ZERO {
                            assert!(
                                water_table.get(tile_pos) < scenario.starting_water_level(),
                                "Water level {} at tile position {} is greater than the starting water level of {}",
                                water_table.get(tile_pos),
                                tile_pos,
                                scenario.starting_water_level()
                            );
                        } else {
                            assert_eq!(
                                water_table.get(tile_pos),
                                scenario.starting_water_level(),
                                "Water level {} at tile position {} is not equal to the starting water level of {}",
                                water_table.get(tile_pos),
                                tile_pos,
                                scenario.starting_water_level()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn precipitation_increase_water_levels() {}

    #[test]
    fn emission_increases_water_levels() {}

    #[test]
    fn root_draw_decreases_water_levels() {}

    #[test]
    fn lateral_flow_conserves_water() {}

    #[test]
    fn lateral_flow_moves_water_from_high_to_low() {}
}
