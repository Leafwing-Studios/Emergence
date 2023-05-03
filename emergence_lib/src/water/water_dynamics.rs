//! Systems that control the movement of water.

use bevy::prelude::*;

use crate::simulation::{
    geometry::{Height, MapGeometry, Volume},
    time::InGameTime,
    weather::CurrentWeather,
};

use super::{WaterConfig, WaterTable};

/// Evaporates water from surface water.
pub(super) fn evaporation(
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

    for tile_pos in map_geometry.valid_tile_positions() {
        // Surface water evaporation
        let total_evaporated = if water_table.surface_water_depth(tile_pos) > Height::ZERO {
            Volume(evaporation_rate)
        } else {
            Volume(evaporation_rate * water_config.soil_evaporation_ratio)
        };

        water_table.remove(tile_pos, total_evaporated);
    }
}

/// Adds water to the water table via rainfall.
pub(super) fn precipitation(
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

    let precipitation_rate = Volume(
        precipitation_per_second * elapsed_time * current_weather.get().precipitation_rate(),
    );

    for tile_pos in map_geometry.valid_tile_positions() {
        water_table.add(tile_pos, precipitation_rate);
    }
}

/// Moves water from one tile to another, according to the relative height of the water table.
pub(super) fn horizontal_water_movement(
    mut water_table: ResMut<WaterTable>,
    water_config: Res<WaterConfig>,
    map_geometry: Res<MapGeometry>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
) {
    let base_water_transfer_amount = water_config.lateral_flow_rate
        / in_game_time.seconds_per_day()
        * fixed_time.period.as_secs_f32();

    for tile_pos in map_geometry.valid_tile_positions() {
        let water_height = water_table.get_height(tile_pos, &map_geometry);
        let neighbors = tile_pos.all_neighbors(&map_geometry);
        for neighbor in neighbors {
            let neighbor_water_height = water_table.get_height(neighbor, &map_geometry);

            let water_transfer = compute_lateral_flow_to_neighbor(
                base_water_transfer_amount,
                &water_config,
                map_geometry.get_height(tile_pos).unwrap(),
                map_geometry.get_height(neighbor).unwrap(),
                water_height,
                neighbor_water_height,
            );

            water_table.remove(tile_pos, water_transfer);
            water_table.add(neighbor, water_transfer);
        }
    }
}

/// Computes how much water should be moved from one tile to another.
#[inline]
fn compute_lateral_flow_to_neighbor(
    base_water_transfer_amount: f32,
    water_config: &WaterConfig,
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
    assert!(water_config.soil_lateral_flow_ratio >= 0.);
    assert!(water_config.soil_lateral_flow_ratio <= 1.);

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
        (true, true) => 1.,
        (false, false) => water_config.soil_lateral_flow_ratio,
        _ => (1. + water_config.soil_lateral_flow_ratio) / 2.,
    };

    let proposed_amount = Volume::from_height(
        delta_water_height * medium_coefficient * base_water_transfer_amount / 2.,
    );
    assert!(proposed_amount >= Volume::ZERO);
    let max_allowable_volume = Volume::from_height(delta_water_height);

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
    use crate::water::WaterPlugin;

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

        let map_geometry = scenario
            .map_shape
            .set_heights(scenario.map_size.map_geometry());
        let water_table = scenario.water_table_strategy.water_table(&map_geometry);

        for &tile_pos in water_table.volume.keys() {
            assert!(
                map_geometry.is_valid(tile_pos),
                "Invalid tile position {} found in water table.",
                tile_pos
            );
        }

        app.insert_resource(water_table);
        app.insert_resource(map_geometry);
        // Override the default water config with one appropriate for testing.
        app.insert_resource(scenario.water_config);
        app.insert_resource(CurrentWeather::new(scenario.weather));

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

        fn water_table(&self, map_geometry: &MapGeometry) -> WaterTable {
            let mut water_table = WaterTable::default();
            for tile_pos in map_geometry.valid_tile_positions() {
                water_table
                    .set_volume(tile_pos, self.starting_water_volume(tile_pos, map_geometry));
            }

            water_table
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
        let mut water_table = WaterTable::default();
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
        let initial_water_table = app.world.resource::<WaterTable>().clone();

        app.update();

        let water_table = app.world.resource::<WaterTable>();
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
                            soil_evaporation_ratio: 0.5,
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
                        lateral_flow_rate: 1000.,
                        soil_lateral_flow_ratio: 0.5,
                        ..WaterConfig::NULL
                    },
                    weather: Weather::Clear,
                    simulated_duration: Duration::from_secs(10),
                };

                let mut app = water_testing_app(scenario);
                let water_table = app.world.resource::<WaterTable>();
                let initial_water = water_table.total_water();

                app.update();

                let water_table = app.world.resource::<WaterTable>();
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
        let mut water_table = app.world.resource_mut::<WaterTable>();
        water_table.add(TilePos::ZERO, Volume(1.0));

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
    fn lateral_flow_moves_water_from_high_to_low() {
        let water_config = WaterConfig {
            lateral_flow_rate: 1.0,
            ..WaterConfig::NULL
        };

        let water_transferred = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
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
        let water_config = WaterConfig {
            lateral_flow_rate: 1.0,
            ..WaterConfig::NULL
        };

        let water_transferred = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(1.0),
            Height(1.0),
            Height(1.0),
            Height(2.0),
        );

        assert_eq!(water_transferred, Volume::ZERO,)
    }

    #[test]
    fn lateral_flow_does_not_move_water_at_equal_heights() {
        let water_config = WaterConfig {
            lateral_flow_rate: 1.0,
            ..WaterConfig::NULL
        };

        let water_transferred = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(1.0),
            Height(1.0),
            Height(1.0),
            Height(1.0),
        );

        assert_eq!(water_transferred, Volume::ZERO,)
    }

    #[test]
    fn surface_water_flows_faster() {
        let water_config: WaterConfig = WaterConfig {
            lateral_flow_rate: 1.0,
            soil_lateral_flow_ratio: 0.1,
            ..WaterConfig::NULL
        };

        let water_height = Height(2.0);
        let neighbor_water_height = Height(1.0);

        let surface_water_flow = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(0.0),
            Height(0.0),
            water_height,
            neighbor_water_height,
        );

        let subsurface_water_flow = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(2.0),
            Height(2.0),
            water_height,
            neighbor_water_height,
        );

        let surface_to_soil_flow = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(0.0),
            Height(2.0),
            water_height,
            neighbor_water_height,
        );

        let soil_to_surface_flow = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
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
        let water_config: WaterConfig = WaterConfig {
            lateral_flow_rate: 1.0,
            ..WaterConfig::NULL
        };

        let small_height_difference = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
            Height(1.0),
            Height(1.0),
            Height(2.0),
            Height(1.0),
        );

        let large_height_difference = compute_lateral_flow_to_neighbor(
            1.0,
            &water_config,
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
        let water_config: WaterConfig = WaterConfig {
            lateral_flow_rate: 1.0,
            ..WaterConfig::NULL
        };

        let base_water_transfer_amount = 0.1;

        let mut water_height_a = Height(2.0);
        let mut water_height_b = Height(1.0);

        let initial_water = water_height_a + water_height_b;

        let tile_height_a = Height(0.0);
        let tile_height_b = Height(0.0);

        for _ in 0..100 {
            let water_transferred_a_to_b = compute_lateral_flow_to_neighbor(
                base_water_transfer_amount,
                &water_config,
                tile_height_a,
                tile_height_b,
                water_height_a,
                water_height_b,
            );

            let water_transferred_b_to_a = compute_lateral_flow_to_neighbor(
                base_water_transfer_amount,
                &water_config,
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
}