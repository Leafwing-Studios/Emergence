//! Systems that control the movement of water.

use std::ops::Div;

use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashMap};
use derive_more::{Add, Sub};
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    light::{shade::ReceivedLight, Illuminance},
    simulation::{
        geometry::{Height, MapGeometry, TilePos, Volume},
        time::InGameTime,
        weather::CurrentWeather,
    },
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

use super::{ocean::Ocean, FlowVelocity, WaterConfig, WaterDepth, WaterVolume};

/// Evaporates water from surface water.
pub(super) fn evaporation(
    mut terrain_query: Query<(&ReceivedLight, &Id<Terrain>, &WaterDepth, &mut WaterVolume)>,
    terrain_manifest: Res<TerrainManifest>,
    water_config: Res<WaterConfig>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
) {
    let evaporation_per_second = water_config.evaporation_rate.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let evaporation_rate = evaporation_per_second * elapsed_time;

    for (received_light, terrain_id, water_depth, mut water_volume) in terrain_query.iter_mut() {
        let mut evaporation_this_tile =
            Volume(evaporation_rate * received_light.evaporation_ratio());

        if matches!(water_depth, WaterDepth::Underground(..)) {
            let terrain_data = terrain_manifest.get(*terrain_id);
            evaporation_this_tile = evaporation_this_tile * terrain_data.water_evaporation_rate;
        }

        water_volume.remove(evaporation_this_tile);
    }
}

impl ReceivedLight {
    /// The rate at which water evaporates from tiles with this amount of light.
    ///
    /// This is a multiplier on the evaporation rate.
    /// [`Illuminance::BrightlyLit`] should always have a value of 1.0.
    fn evaporation_ratio(&self) -> f32 {
        match self.0 {
            Illuminance::Dark => 0.2,
            Illuminance::DimlyLit => 0.5,
            Illuminance::BrightlyLit => 1.0,
        }
    }
}

/// Adds water to the water table via rainfall.
pub(super) fn precipitation(
    water_config: Res<WaterConfig>,
    in_game_time: Res<InGameTime>,
    fixed_time: Res<FixedTime>,
    current_weather: Res<CurrentWeather>,
    mut water_query: Query<&mut WaterVolume>,
) {
    let precipitation_per_second =
        water_config.precipitation_rate.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();

    let precipitation_rate = Volume(
        precipitation_per_second * elapsed_time * current_weather.get().precipitation_rate(),
    );

    for mut water_volume in water_query.iter_mut() {
        water_volume.add(precipitation_rate);
    }
}

/// The relative rate at which water flows between soil of this type.
///
/// This should be less than 1.0, as 1.0 is the rate at which surface water flows.
#[derive(Component, Clone, Copy, Debug, Add, Sub, PartialEq, Serialize, Deserialize)]
pub struct SoilWaterFlowRate(pub f32);

impl Default for SoilWaterFlowRate {
    fn default() -> Self {
        Self(0.5)
    }
}

impl SoilWaterFlowRate {
    /// The value of this coefficient for water that is above the surface of the soil.
    const SURFACE_WATER: SoilWaterFlowRate = SoilWaterFlowRate(1.0);
}

impl Div<f32> for SoilWaterFlowRate {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        SoilWaterFlowRate(self.0 / rhs)
    }
}

/// Data needed to compute lateral flow.
#[allow(missing_docs)]
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct LateralFlowQuery {
    water_volume: &'static mut WaterVolume,
    flow_velocity: &'static mut FlowVelocity,
    water_depth: &'static WaterDepth,
    terrain_height: &'static Height,
    tile_pos: &'static TilePos,
    soil_water_flow_rate: &'static SoilWaterFlowRate,
}

/// Moves water from one tile to another, according to the relative height of the water table.
pub fn horizontal_water_movement(
    mut terrain_query: Query<LateralFlowQuery>,
    water_config: Res<WaterConfig>,
    map_geometry: Res<MapGeometry>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
    ocean: Res<Ocean>,
) {
    let base_water_transfer_amount = water_config.lateral_flow_rate
        / in_game_time.seconds_per_day()
        * fixed_time.period.as_secs_f32();

    // PERF: it will probably be much faster to store scratch space on components
    // Stores the total volume to be removed from each tile
    let mut addition_map = HashMap::<TilePos, Volume>::default();
    // Stores the total volume to be added to each tile
    let mut removal_map = HashMap::<TilePos, Volume>::default();
    // Stores the net water flow direction for each tile
    let mut flow_direction_map = HashMap::<TilePos, FlowVelocity>::default();

    for tile_pos in map_geometry.valid_tile_positions() {
        addition_map.insert(tile_pos, Volume::ZERO);
        removal_map.insert(tile_pos, Volume::ZERO);
        flow_direction_map.insert(tile_pos, FlowVelocity::ZERO);
    }

    for query_item in terrain_query.iter() {
        let total_available = query_item.water_volume.volume();
        if total_available <= Volume::ZERO {
            continue;
        }

        let water_to_neighbors = proposed_lateral_flow_to_neighbors(
            *query_item.tile_pos,
            base_water_transfer_amount,
            &water_config,
            &map_geometry,
            &terrain_query,
            ocean.height(),
        );

        // Ensure that we divide the water evenly between all neighbors
        // Only transfer as much water as is available.
        let total_proposed = water_to_neighbors
            .values()
            .fold(Volume::ZERO, |acc, proposed| acc + *proposed);
        let actual_water_transfer_ratio = (total_available / total_proposed).min(1.0);

        for (&neighbor, &proposed_water_transfer) in water_to_neighbors.iter() {
            let actual_water_transfer = proposed_water_transfer * actual_water_transfer_ratio;

            addition_map
                .entry(neighbor)
                .and_modify(|v| *v += actual_water_transfer);
            removal_map
                .entry(*query_item.tile_pos)
                .and_modify(|v| *v += actual_water_transfer);

            let direction_to_neighbor = neighbor.main_direction_to(query_item.tile_pos.hex);

            // This map only tracks outward flow, so we don't need to update the neighbor.
            flow_direction_map
                .entry(*query_item.tile_pos)
                .and_modify(|v| {
                    *v += FlowVelocity::from_hex_direction(
                        direction_to_neighbor,
                        actual_water_transfer,
                        &map_geometry,
                    )
                });
        }
    }

    // Flow back in from the ocean tiles
    // Flowing out to ocean tiles is implicitly handled by the above code: missing values are treated as if they are ocean tiles
    if water_config.enable_oceans {
        for tile_pos in map_geometry.ocean_tiles() {
            // Don't bother flowing to and from ocean tiles
            for valid_neighbor in tile_pos.all_valid_neighbors(&map_geometry) {
                let neighbor_entity = map_geometry.get_terrain(valid_neighbor).unwrap();
                let neighbor_query_item = terrain_query.get(neighbor_entity).unwrap();

                let neighbor_tile_height = *neighbor_query_item.terrain_height;
                let neighbor_water_height = neighbor_query_item
                    .water_depth
                    .water_table_height(neighbor_tile_height);
                let neighbor_soil_lateral_flow_ratio = *neighbor_query_item.soil_water_flow_rate;

                let proposed_water_transfer = lateral_flow(
                    base_water_transfer_amount,
                    SoilWaterFlowRate(1.0),
                    neighbor_soil_lateral_flow_ratio,
                    Height::ZERO,
                    neighbor_tile_height,
                    ocean.height(),
                    neighbor_water_height,
                );

                if proposed_water_transfer > Volume::ZERO {
                    addition_map.entry(valid_neighbor).and_modify(|v| {
                        *v += proposed_water_transfer;
                    });
                }
            }
        }
    }

    for (tile_pos, volume) in addition_map {
        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let mut query_item = terrain_query.get_mut(terrain_entity).unwrap();
        query_item.water_volume.add(volume);
    }

    for (tile_pos, volume) in removal_map {
        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let mut query_item = terrain_query.get_mut(terrain_entity).unwrap();
        query_item.water_volume.remove(volume);
    }

    for (tile_pos, flow_velocity) in flow_direction_map {
        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let mut query_item = terrain_query.get_mut(terrain_entity).unwrap();
        *query_item.flow_velocity = flow_velocity;
    }
}

/// Computes how much water should be removed from one tile to its neigbors.
///
/// This does not take into account the actual available volume of water in the tile.
#[inline]
#[must_use]
fn proposed_lateral_flow_to_neighbors(
    tile_pos: TilePos,
    base_water_transfer_amount: f32,
    water_config: &WaterConfig,
    map_geometry: &MapGeometry,
    terrain_query: &Query<LateralFlowQuery>,
    ocean_height: Height,
) -> HashMap<TilePos, Volume> {
    let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
    let query_item = terrain_query.get(terrain_entity).unwrap();
    let soil_lateral_flow_ratio = *query_item.soil_water_flow_rate;
    let tile_height = *query_item.terrain_height;
    let water_height = query_item.water_depth.water_table_height(tile_height);

    // Critically, this includes neighbors that are not valid tiles.
    // This is important because we need to be able to transfer water off the edge of the map.
    let neighbors = tile_pos.all_neighbors();
    let mut water_to_neighbors = HashMap::default();

    for neighbor in neighbors {
        // Non-valid neighbors are treated as if they are ocean tiles, and cause water to flow off the edge of the map.
        if !water_config.enable_oceans && !map_geometry.is_valid(neighbor) {
            continue;
        }

        // Neighbor is not an ocean tile
        let proposed_water_transfer =
            if let Some(neigbor_entity) = map_geometry.get_terrain(neighbor) {
                let neighbor_query_item = terrain_query.get(neigbor_entity).unwrap();

                let neighbor_soil_lateral_flow_ratio = *neighbor_query_item.soil_water_flow_rate;
                let neighbor_tile_height = *neighbor_query_item.terrain_height;

                let neighbor_water_height = neighbor_query_item
                    .water_depth
                    .water_table_height(neighbor_tile_height);

                lateral_flow(
                    base_water_transfer_amount,
                    soil_lateral_flow_ratio,
                    neighbor_soil_lateral_flow_ratio,
                    tile_height,
                    neighbor_tile_height,
                    water_height,
                    neighbor_water_height,
                )
            // Neigbor is an ocean tile
            } else {
                lateral_flow(
                    base_water_transfer_amount,
                    soil_lateral_flow_ratio,
                    SoilWaterFlowRate::SURFACE_WATER,
                    tile_height,
                    Height::ZERO,
                    water_height,
                    ocean_height,
                )
            };

        water_to_neighbors.insert(neighbor, proposed_water_transfer);
    }

    water_to_neighbors
}

/// Computes how much water should be moved from one tile to another.
#[inline]
#[must_use]
fn lateral_flow(
    base_water_transfer_amount: f32,
    soil_lateral_flow_ratio: SoilWaterFlowRate,
    neighbor_soil_lateral_flow_ratio: SoilWaterFlowRate,
    tile_height: Height,
    neighbor_tile_height: Height,
    water_height: Height,
    neighbor_water_height: Height,
) -> Volume {
    assert!(base_water_transfer_amount >= 0.);
    assert!(water_height >= Height::ZERO);
    assert!(neighbor_water_height >= Height::ZERO);
    assert!(tile_height >= Height::ZERO);
    assert!(neighbor_tile_height >= Height::ZERO);

    // If the water is higher than the neighbor, move water from the tile to the neighbor
    // at a rate proportional to the height difference.
    // If the water is lower than the neighbor, the flow direction is reversed.
    // The rate is halved as we do the same computation in both directions.

    let delta_water_height = water_height - neighbor_water_height;

    // Water can only flow downhill
    if delta_water_height <= Height::ZERO {
        return Volume::ZERO;
    }

    let surface_water_present = water_height > tile_height;
    let neighbor_surface_water_present = neighbor_water_height > neighbor_tile_height;

    // Water flows more easily between tiles that are both flooded.
    let medium_coefficient = match (surface_water_present, neighbor_surface_water_present) {
        (true, true) => SoilWaterFlowRate::SURFACE_WATER,
        (false, false) => (soil_lateral_flow_ratio + neighbor_soil_lateral_flow_ratio) / 2.,
        (true, false) => (SoilWaterFlowRate::SURFACE_WATER + soil_lateral_flow_ratio) / 2.,
        (false, true) => (SoilWaterFlowRate::SURFACE_WATER + neighbor_soil_lateral_flow_ratio) / 2.,
    }
    .0;

    let proposed_amount = Volume::from_height(
        delta_water_height * medium_coefficient * base_water_transfer_amount / 2.,
    );
    assert!(proposed_amount >= Volume::ZERO);
    // We don't want to move more than half the difference in water height.
    // We *could* move up to the full difference, but that would cause the water to oscillate.
    let max_allowable_volume = Volume::from_height(delta_water_height / 2.);

    proposed_amount.min(max_allowable_volume)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use emergence_macros::IterableEnum;
    use rand::Rng;

    use crate as emergence_lib;
    use crate::enum_iter::IterableEnum;
    use crate::simulation::geometry::TilePos;
    use crate::simulation::time::advance_in_game_time;
    use crate::simulation::weather::{Weather, WeatherPlugin};
    use crate::simulation::SimulationSet;
    use crate::water::{SoilWaterCapacity, WaterPlugin};

    use super::*;
    use crate::structures::Landmark;

    #[derive(Debug, Clone, Copy)]
    struct Scenario {
        water_config: WaterConfig,
        water_table_strategy: WaterTableStrategy,
        map_size: MapSize,
        map_shape: MapShape,
        weather: Weather,
        simulated_duration: Duration,
    }

    /// The smallest amount of water that we care about in these tests.
    const EPSILON: Volume = Volume(0.001);

    /// The smallest height difference that we care about in these tests.
    const EPSILON_HEIGHT: Height = Height(0.001);

    fn water_testing_app(scenario: Scenario) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugin(WaterPlugin)
            .add_plugin(WeatherPlugin)
            .init_resource::<InGameTime>()
            .add_system(
                advance_in_game_time
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );

        let mut map_geometry = scenario
            .map_shape
            .set_heights(scenario.map_size.map_geometry());

        // Override the default water config with one appropriate for testing.
        app.insert_resource(scenario.water_config);
        app.insert_resource(CurrentWeather::new(scenario.weather));

        // Spawn terrain
        for tile_pos in map_geometry
            .valid_tile_positions()
            .collect::<Vec<TilePos>>()
        {
            let terrain_entity = app
                .world
                .spawn((
                    Id::<Terrain>::from_name("test".to_string()),
                    tile_pos,
                    ReceivedLight::default(),
                ))
                .id();
            map_geometry.add_terrain(tile_pos, terrain_entity)
        }

        app.insert_resource(map_geometry);

        // Spawn emitter
        app.world.spawn((Landmark, TilePos::ZERO));

        // Our key systems are run in the fixed update schedule.
        // In order to ensure that the water table is updated in our tests, we must advance the fixed time.
        let mut fixed_time = app.world.resource_mut::<FixedTime>();
        fixed_time.tick(scenario.simulated_duration);

        app
    }

    /// Controls the initial water level of the map.
    #[derive(Debug, IterableEnum, Clone, Copy)]
    enum WaterTableStrategy {
        /// No water.
        Dry,
        /// Half a tile of water.
        DepthHalf,
        /// One tile of water.
        DepthOne,
        /// The water table is at the same height as the surface.
        Saturated,
        /// The water table is one tile above the surface.
        Flooded,
    }

    impl WaterTableStrategy {
        fn starting_water_volume(&self, tile_pos: TilePos, map_geometry: &MapGeometry) -> Volume {
            match self {
                WaterTableStrategy::Dry => Volume(0.),
                WaterTableStrategy::DepthHalf => Volume(0.5),
                WaterTableStrategy::DepthOne => Volume(1.),
                WaterTableStrategy::Saturated => {
                    Volume::from_height(map_geometry.get_height(tile_pos).unwrap())
                }
                WaterTableStrategy::Flooded => {
                    Volume::from_height(map_geometry.get_height(tile_pos).unwrap() + Height(1.))
                }
            }
        }
    }

    /// The size of the test map.
    #[derive(Debug, IterableEnum, Clone, Copy)]
    enum MapSize {
        /// Radius 0 map.
        OneTile,
        /// Radius 3 map.
        Tiny,
    }

    impl MapSize {
        fn map_geometry(&self) -> MapGeometry {
            match self {
                MapSize::OneTile => MapGeometry::new(0),
                MapSize::Tiny => MapGeometry::new(3),
            }
        }
    }

    /// The shape of the test map.
    #[derive(Debug, IterableEnum, Clone, Copy)]
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
                    // Make sure we don't end up with negative heights.
                    MapShape::Sloped => Height(tile_pos.x.max(0) as f32),
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
        let tile_pos = TilePos::new(0, 0);
        water_table.set_volume(tile_pos, Volume(1.0));
        assert_eq!(water_table.get_volume(tile_pos), Volume(1.0));

        water_table.add(tile_pos, Volume(1.0));
        assert_eq!(water_table.get_volume(tile_pos), Volume(2.0));

        water_table.remove(tile_pos, Volume(1.0));
        assert_eq!(water_table.get_volume(tile_pos), Volume(1.0));

        water_table.remove(tile_pos, Volume(1.0));
        assert_eq!(water_table.get_volume(tile_pos), Volume(0.0));

        water_table.remove(tile_pos, Volume(1.0));
        assert_eq!(water_table.get_volume(tile_pos), Volume(0.0));
    }

    #[test]
    fn water_testing_applies_water_dynamics() {
        let scenario = Scenario {
            map_size: MapSize::Tiny,
            map_shape: MapShape::Flat,
            water_table_strategy: WaterTableStrategy::DepthOne,
            water_config: WaterConfig::IN_GAME,
            weather: Weather::Cloudy,
            simulated_duration: Duration::from_secs(1),
        };

        let mut app = water_testing_app(scenario);

        app.update();

        assert!(
            water_table != &initial_water_table,
            "Water table was not updated in {:?}",
            scenario
        );
    }

    #[test]
    fn evaporation_decreases_water_levels() {
        for map_size in MapSize::variants() {
            for map_shape in MapShape::variants() {
                for water_table_strategy in WaterTableStrategy::variants() {
                    let scenario = Scenario {
                        map_size,
                        map_shape,
                        water_table_strategy,
                        water_config: WaterConfig {
                            evaporation_rate: Height(1.0),
                            ..WaterConfig::NULL
                        },
                        weather: Weather::Clear,
                        simulated_duration: Duration::from_secs(1),
                    };

                    let mut app = water_testing_app(scenario);
                    app.update();

                    let water_table = app.world.resource::<WaterTable>();
                    let map_geometry = app.world.resource::<MapGeometry>();

                    for &tile_pos in water_table.volume.keys() {
                        if water_table_strategy.starting_water_volume(tile_pos, &map_geometry)
                            > Volume::ZERO
                        {
                            assert!(
                                water_table.get_volume(tile_pos) < water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                                "Water level {:?} at tile position {} is greater than or equal to the starting water level of {:?} in {:?}",
                                water_table.get_volume(tile_pos),
                                tile_pos,
                                water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                                scenario
                            );
                        } else {
                            assert_eq!(
                                water_table.get_volume(tile_pos),
                                water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                                "Water level {:?} at tile position {} is not equal to the starting water level of {:?} in {:?}",
                                water_table.get_volume(tile_pos),
                                tile_pos,
                                water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                                scenario
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn precipitation_increase_water_levels() {
        for map_size in MapSize::variants() {
            for map_shape in MapShape::variants() {
                for water_table_strategy in WaterTableStrategy::variants() {
                    let scenario = Scenario {
                        map_size,
                        map_shape,
                        water_table_strategy,
                        water_config: WaterConfig {
                            precipitation_rate: Height(1.0),
                            ..WaterConfig::NULL
                        },
                        weather: Weather::Rainy,
                        simulated_duration: Duration::from_secs(1),
                    };

                    let mut app = water_testing_app(scenario);
                    app.update();

                    let water_table = app.world.resource::<WaterTable>();
                    let map_geometry = app.world.resource::<MapGeometry>();

                    for &tile_pos in water_table.volume.keys() {
                        assert!(
                            water_table.get_volume(tile_pos) > water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                            "Water level {:?} at tile position {} is less than the starting water level of {:?} in {:?}",
                            water_table.get_volume(tile_pos),
                            tile_pos,
                            water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                            scenario
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn emission_increases_water_levels() {
        for map_size in MapSize::variants() {
            for water_table_strategy in WaterTableStrategy::variants() {
                let scenario = Scenario {
                    map_size,
                    map_shape: MapShape::Flat,
                    water_table_strategy,
                    water_config: WaterConfig {
                        emission_rate: Volume(1.0),
                        emission_pressure: Height(5.0),
                        lateral_flow_rate: 1000.,
                        ..WaterConfig::NULL
                    },
                    weather: Weather::Clear,
                    simulated_duration: Duration::from_secs(10),
                };

                let mut app = water_testing_app(scenario);
                let initial_water = water_table.total_water();

                app.update();

                let map_geometry = app.world.resource::<MapGeometry>();
                let final_water = water_table.total_water();

                assert!(
                    final_water > initial_water,
                    "Water level {:?} is not greater than the initial water level of {:?} in {:?}",
                    final_water,
                    initial_water,
                    scenario
                );

                for &tile_pos in water_table.volume.keys() {
                    assert!(
                            water_table.get_volume(tile_pos) > water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                            "Water level {:?} at tile position {} is less than or equal to the starting water level of {:?} in {:?}",
                            water_table.get_volume(tile_pos),
                            tile_pos,
                            water_table_strategy.starting_water_volume(tile_pos, &map_geometry),
                            scenario
                        );
                }
            }
        }
    }

    #[test]
    fn lateral_flow_levels_out_hill() {
        let scenario = Scenario {
            map_size: MapSize::Tiny,
            map_shape: MapShape::Bedrock,
            water_table_strategy: WaterTableStrategy::DepthOne,
            water_config: WaterConfig {
                lateral_flow_rate: 1000.,
                ..WaterConfig::NULL
            },
            weather: Weather::Clear,
            simulated_duration: Duration::from_secs(10),
        };

        let mut app = water_testing_app(scenario);
        water_table.add(TilePos::ZERO, Volume(1.0));

        app.update();

        let map_geometry = app.world.resource::<MapGeometry>();

        let average_water_height = water_table.average_height(map_geometry);

        for tile_pos in map_geometry.valid_tile_positions() {
            let height = water_table.get_height(tile_pos, map_geometry);
            assert!(
                height.abs_diff(average_water_height) < EPSILON_HEIGHT,
                "Water level {:?} at tile position {} is not equal to the average water level of {:?}
                The water table is {:?}",
                height,
                tile_pos,
                average_water_height,
                water_table
            )
        }
    }

    #[test]
    fn lateral_flow_levels_out_valley() {
        let scenario = Scenario {
            map_size: MapSize::Tiny,
            map_shape: MapShape::Bedrock,
            water_table_strategy: WaterTableStrategy::DepthOne,
            water_config: WaterConfig {
                lateral_flow_rate: 1000.,
                ..WaterConfig::NULL
            },
            weather: Weather::Clear,
            simulated_duration: Duration::from_secs(10),
        };

        let mut app = water_testing_app(scenario);
        let mut water_table = app.world.resource_mut::<WaterTable>();
        water_table.remove(TilePos::ZERO, Volume(1.0));

        app.update();

        let water_table = app.world.resource::<WaterTable>();
        let map_geometry = app.world.resource::<MapGeometry>();

        let average_water_height = water_table.average_height(map_geometry);

        for tile_pos in map_geometry.valid_tile_positions() {
            let height = water_table.get_height(tile_pos, map_geometry);
            assert!(
                height.abs_diff(average_water_height) < EPSILON_HEIGHT,
                "Water level {:?} at tile position {} is not equal to the average water level of {:?}
                The water table is {:?}",
                height,
                tile_pos,
                average_water_height,
                water_table
            )
        }
    }

    #[test]
    fn doing_nothing_conserves_water() {
        for map_size in MapSize::variants() {
            for map_shape in MapShape::variants() {
                for water_table_strategy in WaterTableStrategy::variants() {
                    let scenario = Scenario {
                        map_size,
                        map_shape,
                        water_table_strategy,
                        water_config: WaterConfig::NULL,
                        weather: Weather::Clear,
                        simulated_duration: Duration::from_secs(3),
                    };

                    let mut app = water_testing_app(scenario);
                    let starting_total_water = app.world.resource::<WaterTable>().total_water();

                    app.update();

                    let final_total_water = app.world.resource::<WaterTable>().total_water();

                    assert!(
                        final_total_water == starting_total_water,
                        "Total water at the end ({:?}) is not equal to the amount of water that we started with ({:?}) in {:?}",
                        final_total_water,
                        starting_total_water,
                        scenario
                    );
                }
            }
        }
    }

    #[test]
    fn lateral_flow_conserves_water() {
        for map_size in MapSize::variants() {
            for map_shape in MapShape::variants() {
                for water_table_strategy in WaterTableStrategy::variants() {
                    let scenario = Scenario {
                        map_size,
                        map_shape,
                        water_table_strategy,
                        water_config: WaterConfig {
                            lateral_flow_rate: 1.0,
                            ..WaterConfig::NULL
                        },
                        weather: Weather::Clear,
                        simulated_duration: Duration::from_secs(5),
                    };

                    let mut app = water_testing_app(scenario);
                    let starting_total_water = app.world.resource::<WaterTable>().total_water();

                    app.update();

                    let final_total_water = app.world.resource::<WaterTable>().total_water();
                    let water_difference = final_total_water.abs_diff(starting_total_water);

                    assert!(
                        water_difference < EPSILON,
                        "Total water at the end ({:?}) is not equal to the amount of water that we started with ({:?}) in {:?}",
                        final_total_water,
                        starting_total_water,
                        scenario
                    );
                }
            }
        }
    }

    #[test]
    fn extremely_high_lateral_flow_conserves_water() {
        for map_size in MapSize::variants() {
            for map_shape in MapShape::variants() {
                for water_table_strategy in WaterTableStrategy::variants() {
                    let scenario = Scenario {
                        map_size,
                        map_shape,
                        water_table_strategy,
                        water_config: WaterConfig {
                            lateral_flow_rate: 9001.0,
                            ..WaterConfig::NULL
                        },
                        weather: Weather::Clear,
                        simulated_duration: Duration::from_secs(5),
                    };

                    let mut app = water_testing_app(scenario);

                    app.update();

                    let water_difference = final_total_water.abs_diff(starting_total_water);

                    assert!(
                        water_difference < EPSILON,
                        "Total water at the end ({:?}) is not equal to the amount of water that we started with ({:?}) in {:?}",
                        final_total_water,
                        starting_total_water,
                        scenario
                    );
                }
            }
        }
    }

    #[test]
    fn lateral_flow_moves_water_from_high_to_low() {
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.7);

        let water_transferred = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(1.0),
            Height(1.0),
            Height(2.0),
            Height(1.0),
        );

        assert!(
            water_transferred > Volume::ZERO,
            "{:?} water was transferred",
            water_transferred
        )
    }

    #[test]
    fn lateral_flow_does_not_move_water_from_low_to_high() {
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.3);

        let water_transferred = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(1.0),
            Height(1.0),
            Height(1.0),
            Height(2.0),
        );

        assert_eq!(water_transferred, Volume::ZERO,)
    }

    #[test]
    fn lateral_flow_does_not_move_water_at_equal_heights() {
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.4);

        let water_transferred = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(1.0),
            Height(1.0),
            Height(1.0),
            Height(1.0),
        );

        assert_eq!(water_transferred, Volume::ZERO,)
    }

    #[test]
    fn surface_water_flows_faster() {
        let water_height = Height(2.0);
        let neighbor_water_height = Height(1.0);
        // This must be less than 1.0 for the test to be valid
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.5);

        let surface_water_flow = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(0.0),
            Height(0.0),
            water_height,
            neighbor_water_height,
        );

        let subsurface_water_flow = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(2.0),
            Height(2.0),
            water_height,
            neighbor_water_height,
        );

        let surface_to_soil_flow = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(0.0),
            Height(2.0),
            water_height,
            neighbor_water_height,
        );

        let soil_to_surface_flow = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(2.0),
            Height(0.0),
            water_height,
            neighbor_water_height,
        );

        assert!(
            surface_water_flow > subsurface_water_flow,
            "Surface water flow ({:?}) is not faster than subsurface water flow ({:?})",
            surface_water_flow,
            subsurface_water_flow
        );

        assert_eq!(surface_to_soil_flow, soil_to_surface_flow);

        assert!(
            surface_to_soil_flow < surface_water_flow,
            "Surface to soil water flow ({:?}) is not slower than surface water flow ({:?})",
            surface_to_soil_flow,
            surface_water_flow
        );
    }

    #[test]
    fn lateral_water_flows_faster_with_larger_height_difference() {
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.1);

        let small_height_difference = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(1.0),
            Height(1.0),
            Height(2.0),
            Height(1.0),
        );

        let large_height_difference = lateral_flow(
            1.0,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            Height(1.0),
            Height(1.0),
            Height(3.0),
            Height(1.0),
        );

        assert!(
            large_height_difference > small_height_difference,
            "Large height difference ({:?}) does not flow faster than small height difference ({:?})",
            large_height_difference,
            small_height_difference
        );
    }

    #[test]
    fn lateral_flow_eventually_equalizes_height_differences() {
        let base_water_transfer_amount = 0.1;
        let soil_lateral_flow_ratio = SoilWaterFlowRate(0.1);

        let mut water_height_a = Height(2.0);
        let mut water_height_b = Height(1.0);

        let initial_water = water_height_a + water_height_b;

        let tile_height_a = Height(0.0);
        let tile_height_b = Height(0.0);

        for _ in 0..100 {
            let water_transferred_a_to_b = lateral_flow(
                base_water_transfer_amount,
                soil_lateral_flow_ratio,
                soil_lateral_flow_ratio,
                tile_height_a,
                tile_height_b,
                water_height_a,
                water_height_b,
            );

            let water_transferred_b_to_a = lateral_flow(
                base_water_transfer_amount,
                soil_lateral_flow_ratio,
                soil_lateral_flow_ratio,
                tile_height_b,
                tile_height_a,
                water_height_b,
                water_height_a,
            );

            println!(
                "Water transferred A to B: {:?}, Water transferred B to A: {:?}",
                water_transferred_a_to_b, water_transferred_b_to_a
            );

            water_height_a += water_transferred_b_to_a.into_height();
            water_height_a -= water_transferred_a_to_b.into_height();

            water_height_b += water_transferred_a_to_b.into_height();
            water_height_b -= water_transferred_b_to_a.into_height();

            let current_water = water_height_a + water_height_b;
            assert!(
                current_water == initial_water,
                "Water was not conserved, starting with {:?} and ending with {:?}",
                initial_water,
                current_water
            );

            println!(
                "Water height A: {:?}, Water height B: {:?}",
                water_height_a, water_height_b
            )
        }

        let water_difference = water_height_a.abs_diff(water_height_b);

        assert!(
            water_difference < EPSILON_HEIGHT,
            "Water levels did not stabilize, ending with a height difference of ({:?}) ",
            water_difference
        );
    }

    #[test]
    fn lateral_flow_should_not_result_in_higher_neighbor() {
        let soil_lateral_flow_ratio = SoilWaterFlowRate(1.0);

        // This is a very high transfer rate, to ensure that the water transfer is maximized
        let base_water_transfer_amount = 1e10;

        let mut water_height_a = Height(2.0);
        let mut water_height_b = Height(1.0);

        let tile_height_a = Height(0.0);
        let tile_height_b = Height(0.0);

        let water_transferred_a_to_b = lateral_flow(
            base_water_transfer_amount,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            tile_height_a,
            tile_height_b,
            water_height_a,
            water_height_b,
        );

        let water_transferred_b_to_a = lateral_flow(
            base_water_transfer_amount,
            soil_lateral_flow_ratio,
            soil_lateral_flow_ratio,
            tile_height_b,
            tile_height_a,
            water_height_b,
            water_height_a,
        );

        water_height_a += water_transferred_b_to_a.into_height();
        water_height_a -= water_transferred_a_to_b.into_height();

        water_height_b += water_transferred_a_to_b.into_height();
        water_height_b -= water_transferred_b_to_a.into_height();

        assert!(
            water_height_a >= water_height_b,
            "Water height A ({:?}) should be greater than or equal to water height B ({:?}) as it started higher.",
            water_height_a,
            water_height_b
        );
    }
}
