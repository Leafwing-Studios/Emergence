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
                height: 10.,
                radius: 10.,
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

/// Returns the height of a cone-shaped hill at the given position.
///
/// The hill is centered at the origin and has a radius of `radius`.
/// The height of the hill is `height` at the center and 0 at the edge.
fn hill(tile_pos: TilePos, hill_settings: &HillSettings) -> f32 {
    let HillSettings { height, radius } = *hill_settings;

    let pos = Vec2::new(tile_pos.hex.x as f32, tile_pos.hex.y as f32);

    let dist = pos.length();
    let height = height * (1.0 - dist / radius).max(0.0);

    Height::MIN.into_world_pos() + height
}

/// A settings struct for [`hill`].
#[derive(Debug, Clone)]
struct HillSettings {
    /// The height of the hill
    height: f32,
    /// The radius of the hill
    radius: f32,
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
