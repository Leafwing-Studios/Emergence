//! Generating starting terrain and organisms
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetState;
use crate::player_interaction::clipboard::ClipboardData;
use crate::simulation::geometry::{Facing, Height, MapGeometry, TilePos};
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::StructureManifest;
use crate::terrain::commands::TerrainCommandsExt;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_assets::UnitHandles;
use crate::units::unit_manifest::UnitManifest;
use crate::units::UnitBundle;
use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::IntoSystemAppConfigs;
use bevy::utils::HashMap;
use hexx::shapes::hexagon;
use hexx::Hex;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Controls world generation strategy
#[derive(Resource, Debug, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub(super) map_radius: u32,
    /// Initial number of ants.
    n_ant: usize,
    /// Initial number of plants.
    n_plant: usize,
    /// Initial number of fungi.
    n_fungi: usize,
    /// Initial number of ant hives.
    n_hive: usize,
    /// Relative probability of generating tiles of each terrain type.
    terrain_weights: HashMap<Id<Terrain>, f32>,
    /// Controls and shape of the hills.
    hill_settings: HillSettings,
    /// Controls the noise added to the terrain heights.
    simplex_settings: SimplexSettings,
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("loam".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("muddy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        GenerationConfig {
            map_radius: 20,
            n_ant: 12,
            n_plant: 12,
            n_fungi: 2,
            n_hive: 1,
            terrain_weights,
            hill_settings: HillSettings {
                center: TilePos::ZERO,
                height: Height(10),
                radius: 10,
            },
            simplex_settings: SimplexSettings {
                frequency: 0.07,
                amplitude: 2.0,
                octaves: 4,
                lacunarity: 2.3,
                gain: 0.5,
                seed: 2378.0,
            },
        }
    }
}

/// Generate the world.
pub(super) struct GenerationPlugin {
    /// Configuration settings for world generation
    pub(super) config: GenerationConfig,
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Generation plugin...");
        app.insert_resource(self.config.clone()).add_systems(
            (generate_terrain, apply_system_buffers, generate_organisms)
                .chain()
                .in_schedule(OnEnter(AssetState::FullyLoaded)),
        );
    }
}

/// Creates the world according to [`GenerationConfig`].
pub(crate) fn generate_terrain(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_weights = &generation_config.terrain_weights;
    let terrain_variants: Vec<Id<Terrain>> = terrain_weights.keys().copied().collect();

    for hex in hexagon(Hex::ZERO, map_geometry.radius) {
        // FIXME: can we not just sample from our terrain_weights directly?
        let &terrain_id = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights.get(terrain_type).unwrap()
            })
            .unwrap();

        let tile_pos = TilePos { hex };
        // Heights are generated in f32 world coordinates to start
        let hex_height = hill(tile_pos, &generation_config.hill_settings)
            + simplex_noise(tile_pos, &generation_config.simplex_settings);

        // And then discretized to the nearest integer height before being used
        let height = Height::from_world_pos(hex_height);

        commands.spawn_terrain(tile_pos, height, terrain_id);
    }
}

/// A settings struct for [`hill`].
#[derive(Debug, Clone)]
struct HillSettings {
    /// The center of the hill
    center: TilePos,
    /// The height of the hill, in height units
    height: Height,
    /// The radius of the hill, in tiles
    radius: u16,
}

/// Returns the height (in world coordinates) of a 2D normal-shaped hill at the given position.
///
/// The hill is centered at the origin and has a radius of `radius`.
/// The height of the hill is `height` at the center and 0 at the edge.
/// The edge corresponds to the 95% confidence interval of a normal distribution,
/// so the radius is 1.96 times larger than the standard deviation.
fn hill(tile_pos: TilePos, hill_settings: &HillSettings) -> f32 {
    let HillSettings {
        center,
        height,
        radius,
    } = *hill_settings;

    // Convert to f32 for computation
    let height = height.into_world_pos();
    let radius = radius as f32;

    /// Returns the value of a normal distribution with the given standard deviation at the given value.
    ///
    /// The mean is 0.
    fn normal(x: f32, standard_deviation: f32) -> f32 {
        let variance = standard_deviation.powi(2);
        (-x.powi(2) / (2. * variance)).exp()
    }

    let distance_from_center = center.distance_to_tile_coordinates(tile_pos);
    if distance_from_center >= radius {
        return 0.;
    }

    let standard_deviation = radius / 1.96;
    // The value at the edge is not exactly 0, but rather the value of the normal distribution at the edge
    // PERF: these values are constant, so we could precompute them
    let minimum_value = normal(radius, standard_deviation);
    let maximum_value = normal(0., standard_deviation);
    let scale_factor = maximum_value - minimum_value;

    // Get the value of the normal distribution
    let value = normal(distance_from_center, standard_deviation);
    // Shift the value so if it is at the edge, it is 0
    let shifted_value = value - minimum_value;
    // Scale the value, so if it is at the center, it is 1
    let scaled_value = shifted_value / scale_factor;
    // Scale the final value to the desired height
    scaled_value * height * Height::STEP_HEIGHT
}

/// A settings struct for [`simplex_noise`].
#[derive(Debug, Clone)]
struct SimplexSettings {
    /// Scale the pos to make it work better with the noise function
    frequency: f32,
    /// Scale the output of the noise function so you can more easily use the number for a height
    amplitude: f32,
    /// How many times will the fbm be sampled?
    octaves: usize,
    /// Smoothing factor
    lacunarity: f32,
    /// Scale the output of the fbm function
    gain: f32,
    /// Arbitary seed that determines the noise function output
    seed: f32,
}

/// Computes the value of the noise function at a given position.
///
/// This can then be used to determine the height of a tile.
fn simplex_noise(tile_pos: TilePos, settings: &SimplexSettings) -> f32 {
    let SimplexSettings {
        frequency,
        amplitude,
        octaves,
        lacunarity,
        gain,
        seed,
    } = *settings;

    let pos = Vec2::new(tile_pos.hex.x as f32, tile_pos.hex.y as f32);

    Height::MIN.into_world_pos()
        + (fbm_simplex_2d_seeded(pos * frequency, octaves, lacunarity, gain, seed) * amplitude)
            .abs()
}

/// Create starting organisms according to [`GenerationConfig`], and randomly place them on
/// passable tiles.
fn generate_organisms(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    tile_query: Query<&TilePos, With<Id<Terrain>>>,
    unit_handles: Res<UnitHandles>,
    unit_manifest: Res<UnitManifest>,
    structure_manifest: Res<StructureManifest>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating organisms...");
    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;
    let n_hive = config.n_hive;

    let n_entities = n_ant + n_plant + n_fungi + n_hive;
    assert!(n_entities <= tile_query.iter().len());

    let mut rng = &mut thread_rng();
    let mut entity_positions: Vec<TilePos> = {
        let possible_positions: Vec<TilePos> = tile_query.iter().copied().collect();

        possible_positions
            .choose_multiple(&mut rng, n_entities)
            .cloned()
            .collect()
    };

    // Ant
    let ant_positions = entity_positions.split_off(entity_positions.len() - n_ant);
    for ant_position in ant_positions {
        commands.spawn(UnitBundle::randomized(
            Id::from_name("ant".to_string()),
            ant_position,
            unit_manifest.get(Id::from_name("ant".to_string())).clone(),
            &unit_handles,
            &map_geometry,
            rng,
        ));
    }

    // Plant
    let plant_positions = entity_positions.split_off(entity_positions.len() - n_plant);
    for position in plant_positions {
        let structure_id = Id::from_name("acacia".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }

    // Fungi
    let fungus_positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    for position in fungus_positions {
        let structure_id = Id::from_name("leuco".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }

    // Hives
    let hive_positions = entity_positions.split_off(entity_positions.len() - n_hive);
    for position in hive_positions {
        let structure_id = Id::from_name("ant_hive".to_string());

        let item = ClipboardData {
            structure_id,
            facing: Facing::default(),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        commands.spawn_randomized_structure(position, item, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hills_have_the_correct_height_at_the_center() {
        let settings = HillSettings {
            center: TilePos::new(3, 7),
            height: Height(2),
            radius: 3,
        };

        let value = hill(settings.center, &settings);
        let computed_height = Height::from_world_pos(value);

        assert_eq!(value, settings.height.into_world_pos());
        assert_eq!(computed_height, settings.height);
    }

    #[test]
    fn hills_have_zero_height_at_edge() {
        let settings = HillSettings {
            center: TilePos::new(0, 0),
            height: Height(2),
            radius: 5,
        };

        let edge = TilePos::new(0, settings.radius as i32);
        let value = hill(edge, &settings);
        let computed_height = Height::from_world_pos(value);

        assert_eq!(value, 0.0);
        assert_eq!(computed_height, Height::MIN);
    }

    #[test]
    fn hills_always_have_positive_height() {
        let map_geometry = MapGeometry::new(10);

        let settings = HillSettings {
            center: TilePos::new(13, -2),
            height: Height(12),
            radius: 4,
        };

        for tile_pos in map_geometry.valid_tile_positions() {
            let value = hill(tile_pos, &settings);
            assert!(value >= 0.0);
        }
    }

    #[test]
    fn hills_are_highest_at_their_center() {
        let map_geometry = MapGeometry::new(10);

        let settings = HillSettings {
            center: TilePos::new(3, 7),
            height: Height(2),
            radius: 13,
        };

        for tile_pos in map_geometry.valid_tile_positions() {
            let value = hill(tile_pos, &settings);
            let computed_height = Height::from_world_pos(value);

            if tile_pos == settings.center {
                assert_eq!(value, settings.height.into_world_pos());
                assert_eq!(computed_height, settings.height);
            } else {
                assert!(value < settings.height.into_world_pos());
                // We have to allow for discretization, so we can't assert that
                // the computed height is strictly less than the desired height.
                assert!(computed_height <= settings.height);
            }
        }
    }

    #[test]
    fn hills_are_lowest_at_their_edge() {
        let map_geometry = MapGeometry::new(10);

        let settings = HillSettings {
            center: TilePos::new(4, 2),
            height: Height(2),
            radius: 5,
        };

        // Hills are defined to have a height of 0 at their edge.
        let height_at_edge = 0.;

        for tile_pos in map_geometry.valid_tile_positions() {
            let distance_from_center = (tile_pos - settings.center).hex.length();
            let height = hill(tile_pos, &settings);

            if distance_from_center >= settings.radius as i32 {
                assert_eq!(height, height_at_edge);
            } else {
                assert!(height > height_at_edge);
            }
        }
    }

    #[test]
    fn hills_are_symmetric() {
        let map_geometry = MapGeometry::new(10);

        let settings = HillSettings {
            center: TilePos::new(3, -2),
            height: Height(5),
            radius: 2,
        };

        for tile_pos in map_geometry.valid_tile_positions() {
            let value = hill(tile_pos, &settings);
            let relative_position = tile_pos - settings.center;

            let symmetric_position = settings.center - relative_position;
            let symmetric_value = hill(symmetric_position, &settings);

            assert_eq!(value, symmetric_value);
        }
    }

    #[test]
    fn hills_are_monotonic_from_edge_to_peak() {
        let settings = HillSettings {
            center: TilePos::new(0, 0),
            height: Height(3),
            radius: 5,
        };

        // Draw a line from the center to the edge of the hill.
        let starting_hex = settings.center.hex;
        let ending_hex = starting_hex + Hex::new(1, 0) * settings.radius as i32;

        let line_of_hexes = starting_hex.line_to(ending_hex);

        let mut previous_value = settings.height.into_world_pos();

        // Walk down the hill, checking that the height is decreasing.
        for hex in line_of_hexes {
            let tile_pos = TilePos { hex };
            let value = hill(tile_pos, &settings);

            assert!(value <= previous_value);

            previous_value = value;
        }
    }
}
