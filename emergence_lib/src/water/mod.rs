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
#[derive(Resource, Debug, Clone, Copy)]
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
    lateral_flow_rate: f32,
    /// The relative rate at which water moves horizontally through soil.
    soil_lateral_flow_ratio: f32,
}

impl WaterConfig {
    /// The default configuration for in-game water behavior.
    const IN_GAME: Self = Self {
        evaporation_rate: Height(0.1),
        soil_evaporation_ratio: 0.5,
        precipitation_rate: Height(0.1),
        emission_rate: Height(100.0),
        root_draw_rate: Height(0.1),
        lateral_flow_rate: 10.0,
        soil_lateral_flow_ratio: 0.5,
    };

    /// A configuration that disables all water behavior.
    #[allow(dead_code)]
    const NULL: Self = Self {
        evaporation_rate: Height(0.0),
        soil_evaporation_ratio: 0.0,
        precipitation_rate: Height(0.0),
        emission_rate: Height(0.0),
        root_draw_rate: Height(0.0),
        lateral_flow_rate: 0.0,
        soil_lateral_flow_ratio: 0.0,
    };
}

/// A plugin that handles water movement and behavior.
pub(super) struct WaterPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WaterSet {
    VerticalWaterMovement,
    HorizontalWaterMovement,
    UpdateGeometry,
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
                        WaterSet::UpdateGeometry,
                    )
                        .in_set(SimulationSet)
                        .chain(),
                )
                .add_systems(
                    (
                        produce_water_from_emitters,
                        precipitation,
                        draw_water_from_roots,
                        evaporation,
                    )
                        .chain()
                        .in_set(WaterSet::VerticalWaterMovement),
                )
                .add_system(horizontal_water_movement.in_set(WaterSet::HorizontalWaterMovement))
                .add_systems(
                    (produce_water_from_emitters, draw_water_from_roots)
                        .in_set(WaterSet::VerticalWaterMovement),
                )
                .add_system(update_surface_water_map_geometry.in_set(WaterSet::UpdateGeometry));
        });
    }
}

/// The height of the water.
///
/// This can be underground, at ground level, or above ground.
/// If it is above ground, it will pool on top of the tile it is on.
#[derive(Resource, Default, PartialEq, Clone, Debug)]
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

    /// Computes the average height of the water table.
    pub(crate) fn average_height(&self, map_geometry: &MapGeometry) -> Height {
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
    let base_water_transfer_amount = water_config.lateral_flow_rate
        / in_game_time.seconds_per_day()
        * fixed_time.period.as_secs_f32();

    for tile_pos in map_geometry.valid_tile_positions() {
        let water_height = water_table.get(tile_pos);
        let neighbors = tile_pos.all_neighbors(&map_geometry);
        for neighbor in neighbors {
            let neighbor_water_height = water_table.get(neighbor);

            let water_transfer = compute_lateral_flow_to_neighbor(
                base_water_transfer_amount,
                &water_config,
                map_geometry.get_height(tile_pos).unwrap(),
                map_geometry.get_height(neighbor).unwrap(),
                water_height,
                neighbor_water_height,
            );

            water_table.subtract(tile_pos, water_transfer);
            water_table.add(neighbor, water_transfer);
        }
    }
}

#[inline]
fn compute_lateral_flow_to_neighbor(
    base_water_transfer_amount: f32,
    water_config: &WaterConfig,
    tile_height: Height,
    neighbor_tile_height: Height,
    water_height: Height,
    neighbor_water_height: Height,
) -> Height {
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
        return Height::ZERO;
    }

    let surface_water_present = water_height > tile_height;
    let neighbor_surface_water_present = neighbor_water_height > neighbor_tile_height;

    // Water flows more easily between tiles that are both flooded.
    let medium_coefficient = match (surface_water_present, neighbor_surface_water_present) {
        (true, true) => 1.,
        (false, false) => water_config.soil_lateral_flow_ratio,
        _ => (1. + water_config.soil_lateral_flow_ratio) / 2.,
    };

    let proposed_amount = delta_water_height * medium_coefficient * base_water_transfer_amount / 2.;
    assert!(proposed_amount >= Height::ZERO);

    let final_amount = proposed_amount.min(delta_water_height);
    assert!(final_amount <= delta_water_height);

    final_amount
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use emergence_macros::IterableEnum;
    use rand::Rng;

    use crate as emergence_lib;
    use crate::asset_management::manifest::Id;
    use crate::construction::ConstructionStrategy;
    use crate::crafting::components::ActiveRecipe;
    use crate::enum_iter::IterableEnum;
    use crate::simulation::time::advance_in_game_time;
    use crate::simulation::weather::{Weather, WeatherPlugin};
    use crate::structures::structure_manifest::{
        Structure, StructureData, StructureKind, StructureManifest,
    };
    use crate::structures::{Footprint, Landmark};

    use super::roots::RootZone;
    use super::*;

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
    const EPSILON: Height = Height(0.001);

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

        for &tile_pos in water_table.height.keys() {
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

        // Spawn something with roots
        let mut structure_manifest = StructureManifest::default();
        structure_manifest.insert(
            "test_plant".to_string(),
            StructureData {
                organism_variety: None,
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::NONE,
                },
                construction_strategy: ConstructionStrategy::Landmark,
                max_workers: 1,
                footprint: Footprint::default(),
                root_zone: Some(RootZone {
                    radius: 1,
                    max_depth: Height(1.),
                }),
                passable: false,
            },
        );

        app.insert_resource(structure_manifest);
        app.world.spawn((
            TilePos::ZERO,
            Id::<Structure>::from_name("test_plant".to_string()),
        ));

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
        fn starting_water_level(&self, tile_pos: TilePos, map_geometry: &MapGeometry) -> Height {
            match self {
                WaterTableStrategy::Dry => Height(0.),
                WaterTableStrategy::DepthHalf => Height(0.5),
                WaterTableStrategy::DepthOne => Height(1.),
                WaterTableStrategy::Saturated => map_geometry.get_height(tile_pos).unwrap(),
                WaterTableStrategy::Flooded => {
                    map_geometry.get_height(tile_pos).unwrap() + Height(1.)
                }
            }
        }

        fn water_table(&self, map_geometry: &MapGeometry) -> WaterTable {
            let mut water_table = WaterTable::default();
            for tile_pos in map_geometry.valid_tile_positions() {
                water_table.set(tile_pos, self.starting_water_level(tile_pos, map_geometry));
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

                    for &tile_pos in water_table.height.keys() {
                        if water_table_strategy.starting_water_level(tile_pos, &map_geometry)
                            > Height::ZERO
                        {
                            assert!(
                                water_table.get(tile_pos) < water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                                "Water level {:?} at tile position {} is greater than or equal to the starting water level of {:?} in {:?}",
                                water_table.get(tile_pos),
                                tile_pos,
                                water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                                scenario
                            );
                        } else {
                            assert_eq!(
                                water_table.get(tile_pos),
                                water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                                "Water level {:?} at tile position {} is not equal to the starting water level of {:?} in {:?}",
                                water_table.get(tile_pos),
                                tile_pos,
                                water_table_strategy.starting_water_level(tile_pos, &map_geometry),
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

                    for &tile_pos in water_table.height.keys() {
                        assert!(
                            water_table.get(tile_pos) > water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                            "Water level {:?} at tile position {} is less than the starting water level of {:?} in {:?}",
                            water_table.get(tile_pos),
                            tile_pos,
                            water_table_strategy.starting_water_level(tile_pos, &map_geometry),
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
                        emission_rate: Height(1.0),
                        lateral_flow_rate: 10.,
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

                for &tile_pos in water_table.height.keys() {
                    assert!(
                            water_table.get(tile_pos) > water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                            "Water level {:?} at tile position {} is less than or equal to the starting water level of {:?} in {:?}",
                            water_table.get(tile_pos),
                            tile_pos,
                            water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                            scenario
                        );
                }
            }
        }
    }

    #[test]
    fn root_draw_decreases_water_levels() {
        for map_size in MapSize::variants() {
            for water_table_strategy in WaterTableStrategy::variants() {
                let scenario = Scenario {
                    map_size,
                    map_shape: MapShape::Flat,
                    water_table_strategy,
                    water_config: WaterConfig {
                        root_draw_rate: Height(1.0),
                        lateral_flow_rate: 10.,
                        soil_lateral_flow_ratio: 0.5,
                        ..WaterConfig::NULL
                    },
                    weather: Weather::Clear,
                    simulated_duration: Duration::from_secs(10),
                };

                let mut app = water_testing_app(scenario);
                let water_table = app.world.resource::<WaterTable>();
                let initial_water = water_table.total_water();
                if initial_water == Height::ZERO {
                    continue;
                }

                app.update();

                let water_table = app.world.resource::<WaterTable>();
                let map_geometry = app.world.resource::<MapGeometry>();

                let final_water = water_table.total_water();

                assert!(
                    final_water < initial_water,
                    "Water level {:?} is not less than the initial water level of {:?} in {:?}",
                    final_water,
                    initial_water,
                    scenario
                );

                for &tile_pos in water_table.height.keys() {
                    assert!(
                            water_table.get(tile_pos) < water_table_strategy.starting_water_level(tile_pos, &map_geometry),
                            "Water level {:?} at tile position {} is greater than the starting water level of {:?} in {:?}",
                            water_table.get(tile_pos),
                            tile_pos,
                            water_table_strategy.starting_water_level(tile_pos, &map_geometry),
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
                lateral_flow_rate: 10.,
                ..WaterConfig::NULL
            },
            weather: Weather::Clear,
            simulated_duration: Duration::from_secs(10),
        };

        let mut app = water_testing_app(scenario);
        let mut water_table = app.world.resource_mut::<WaterTable>();
        water_table.add(TilePos::ZERO, Height(1.0));

        app.update();

        let water_table = app.world.resource::<WaterTable>();
        let map_geometry = app.world.resource::<MapGeometry>();

        let average_water_height = water_table.average_height(map_geometry);

        for tile_pos in map_geometry.valid_tile_positions() {
            let height = water_table.get(tile_pos);
            assert!(
                height.abs_diff(average_water_height) < EPSILON,
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
                lateral_flow_rate: 10.,
                ..WaterConfig::NULL
            },
            weather: Weather::Clear,
            simulated_duration: Duration::from_secs(10),
        };

        let mut app = water_testing_app(scenario);
        let mut water_table = app.world.resource_mut::<WaterTable>();
        water_table.subtract(TilePos::ZERO, Height(1.0));

        app.update();

        let water_table = app.world.resource::<WaterTable>();
        let map_geometry = app.world.resource::<MapGeometry>();

        let average_water_height = water_table.average_height(map_geometry);

        for tile_pos in map_geometry.valid_tile_positions() {
            let height = water_table.get(tile_pos);
            assert!(
                height.abs_diff(average_water_height) < EPSILON,
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
            water_transferred > Height::ZERO,
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

        assert_eq!(water_transferred, Height::ZERO,)
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

        assert_eq!(water_transferred, Height::ZERO,)
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

            water_height_a += water_transferred_b_to_a;
            water_height_a -= water_transferred_a_to_b;

            water_height_b += water_transferred_a_to_b;
            water_height_b -= water_transferred_b_to_a;

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
            water_difference < EPSILON,
            "Water levels did not stabilize, ending with a height difference of ({:?}) ",
            water_difference
        );
    }
}
