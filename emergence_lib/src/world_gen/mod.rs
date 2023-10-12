//! Generating starting terrain and organisms
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetState;
use crate::structures::structure_manifest::Structure;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_manifest::Unit;
use crate::utils::noise::SimplexSettings;
use crate::world_gen::structure_generation::generate_structures;
use crate::world_gen::unit_generation::{generate_units, randomize_starting_organisms};

use crate::world_gen::terrain_generation::{
    generate_landmarks, generate_terrain, initialize_water_table,
};

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_framepace::{FramepaceSettings, Limiter};

mod structure_generation;
mod terrain_generation;
mod unit_generation;

/// Generate the world.
pub(super) struct GenerationPlugin {
    /// Configuration settings for world generation
    pub(super) config: GenerationConfig,
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Generation plugin...");
        app.add_state::<WorldGenState>()
            .insert_resource(self.config.clone())
            .add_systems(
                Update,
                (
                    generate_terrain,
                    apply_deferred,
                    generate_landmarks,
                    initialize_water_table,
                    apply_deferred,
                    generate_structures,
                    apply_deferred,
                    generate_units,
                    apply_deferred,
                    randomize_starting_organisms,
                )
                    .chain()
                    .in_schedule(OnEnter(WorldGenState::Generating)),
            )
            .add_systems(
                PreUpdate,
                WorldGenState::manage_state.run_if(|world_gen_state: Res<State<WorldGenState>>| {
                    world_gen_state.get() != WorldGenState::Complete
                }),
            );
    }
}

/// Tracks world generation progress.
#[derive(Default, States, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldGenState {
    /// The world is waiting to be generated.
    #[default]
    Waiting,
    /// The world is being generated.
    Generating,
    /// The world has been generated, but we're simulating for a while to let things settle.
    BurningIn,
    /// The world has been generated.
    Complete,
}

impl WorldGenState {
    /// A system that advances the world generation state machine.
    fn manage_state(
        mut number_of_burn_in_ticks: Local<u32>,
        generation_config: Res<GenerationConfig>,
        world_gen_state: Res<State<WorldGenState>>,
        mut next_world_gen_state: ResMut<NextState<WorldGenState>>,
        mut maybe_frame_pace_settings: Option<ResMut<FramepaceSettings>>,
        maybe_asset_state: Option<Res<State<AssetState>>>,
    ) {
        match world_gen_state.get() {
            WorldGenState::Waiting => {
                if let Some(frame_pace_settings) = maybe_frame_pace_settings.as_mut() {
                    // Don't limit the tick rate while generating the world
                    if !matches!(frame_pace_settings.limiter, Limiter::Off) {
                        frame_pace_settings.limiter = Limiter::Off;
                    }
                }

                if let Some(asset_state) = maybe_asset_state {
                    if asset_state.get() == AssetState::FullyLoaded {
                        next_world_gen_state.set(WorldGenState::Generating);
                    }
                } else {
                    next_world_gen_state.set(WorldGenState::Generating);
                }
            }
            WorldGenState::Generating => {
                next_world_gen_state.set(WorldGenState::BurningIn);
            }
            WorldGenState::BurningIn => {
                *number_of_burn_in_ticks += 1;

                if *number_of_burn_in_ticks > generation_config.number_of_burn_in_ticks {
                    info!("Burn in complete.");

                    // Resume limiting the tick rate
                    if let Some(mut frame_pace_settings) = maybe_frame_pace_settings {
                        frame_pace_settings.limiter = Limiter::Auto;
                    }

                    next_world_gen_state.set(WorldGenState::Complete);
                } else {
                    info!(
                        "Simulating the generated world to let it stabilize: {}/{}",
                        *number_of_burn_in_ticks, generation_config.number_of_burn_in_ticks
                    );
                }
            }
            WorldGenState::Complete => (),
        }
    }
}

/// Controls world generation strategy
#[derive(Resource, Debug, Clone)]
pub struct GenerationConfig {
    /// The seed used to generate the world.
    pub seed: u64,
    /// Radius of the map.
    pub(super) map_radius: u32,
    /// How long to simulate the world before starting the game.
    number_of_burn_in_ticks: u32,
    /// Chance that each tile contains a landmark of the given type.
    landmark_chances: HashMap<Id<Structure>, f32>,
    /// Chance that each tile contains a unit of the given type.
    unit_chances: HashMap<Id<Unit>, f32>,
    /// Chance that each tile contains a structure of the given type.
    structure_chances: HashMap<Id<Structure>, f32>,
    /// Relative probability of generating tiles of each terrain type.
    terrain_weights: HashMap<Id<Terrain>, f32>,
    /// Controls the noise added to produce the larger land forms.
    low_frequency_noise: SimplexSettings,
    /// Controls the noise added to the terrain heights.
    high_frequency_noise: SimplexSettings,
}

impl GenerationConfig {
    /// The default world generation configuration.
    pub fn standard() -> Self {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("grassy".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("swampy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let mut landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        landmark_chances.insert(Id::from_name("spring".to_string()), 5e-4);

        let mut unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();
        unit_chances.insert(Id::from_name("basket_crab".to_string()), 1e-2);

        let mut structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        structure_chances.insert(Id::from_name("ant_hive".to_string()), 1e-3);
        structure_chances.insert(Id::from_name("acacia".to_string()), 2e-2);
        structure_chances.insert(Id::from_name("leuco".to_string()), 1e-2);
        structure_chances.insert(Id::from_name("tide_weed".to_string()), 3e-2);

        GenerationConfig {
            seed: 0,
            map_radius: 30,
            number_of_burn_in_ticks: 0,
            unit_chances,
            landmark_chances,
            structure_chances,
            terrain_weights,
            low_frequency_noise: SimplexSettings {
                frequency: 1e-2,
                amplitude: 8.0,
                octaves: 4,
                lacunarity: 1.,
                gain: 0.5,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 1.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
            },
        }
    }

    /// A small flat map for testing.
    pub fn flat() -> Self {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("grassy".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("swampy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let mut landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        landmark_chances.insert(Id::from_name("spring".to_string()), 5e-4);

        let mut unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();
        unit_chances.insert(Id::from_name("basket_crab".to_string()), 1e-2);

        let mut structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        structure_chances.insert(Id::from_name("ant_hive".to_string()), 1e-3);
        structure_chances.insert(Id::from_name("acacia".to_string()), 2e-2);
        structure_chances.insert(Id::from_name("leuco".to_string()), 1e-2);
        structure_chances.insert(Id::from_name("tide_weed".to_string()), 3e-2);

        GenerationConfig {
            seed: 0,
            map_radius: 10,
            number_of_burn_in_ticks: 0,
            unit_chances,
            landmark_chances,
            structure_chances,
            terrain_weights,
            low_frequency_noise: SimplexSettings {
                frequency: 1e-2,
                amplitude: 0.0,
                octaves: 4,
                lacunarity: 1.,
                gain: 0.5,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 0.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
            },
        }
    }

    /// A tiny world gen config for testing.
    pub fn testing() -> Self {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("grassy".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let mut landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        landmark_chances.insert(Id::from_name("simple_landmark".to_string()), 1e-1);

        let mut unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();
        unit_chances.insert(Id::from_name("simple_unit".to_string()), 1.);

        let mut structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        structure_chances.insert(Id::from_name("simple_structure".to_string()), 1e-1);
        structure_chances.insert(Id::from_name("passable_structure".to_string()), 1e-1);

        GenerationConfig {
            seed: 0,
            map_radius: 3,
            number_of_burn_in_ticks: 0,
            unit_chances,
            landmark_chances,
            structure_chances,
            terrain_weights,
            low_frequency_noise: SimplexSettings {
                frequency: 1e-2,
                amplitude: 8.0,
                octaves: 4,
                lacunarity: 1.,
                gain: 0.5,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 1.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::asset_management::manifest::DummyManifestPlugin;
    use crate::geometry::{MapGeometry, VoxelPos};
    use crate::simulation::rng::GlobalRng;
    use crate::water::WaterConfig;

    use super::*;

    #[test]
    fn can_generate_terrain() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(Startup, generate_terrain);

        app.update();
    }

    #[test]
    fn can_generate_organisms() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(Startup, (generate_terrain, generate_structures).chain());

        app.update();
    }

    #[test]
    fn units_are_on_top_of_empty_ground() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(
            Startup,
            (generate_terrain, generate_structures, generate_landmarks).chain(),
        );

        app.update();

        let map_geometry = app.world.resource::<MapGeometry>().clone();
        let walkable_voxels = map_geometry.walkable_voxels();

        let mut unit_query = app.world.query_filtered::<&VoxelPos, With<Id<Unit>>>();

        for &voxel_pos in unit_query.iter(&app.world) {
            let terrain_height = map_geometry.get_height(voxel_pos.hex).unwrap();
            assert_eq!(voxel_pos.height, terrain_height.above());
            assert!(map_geometry.is_voxel_clear(voxel_pos).is_ok());
            assert!(walkable_voxels.contains(&voxel_pos));
        }
    }

    #[test]
    fn structures_are_above_ground() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(
            Startup,
            (generate_terrain, generate_structures, generate_landmarks).chain(),
        );

        app.update();

        let map_geometry = app.world.resource::<MapGeometry>().clone();
        let mut structure_query = app.world.query_filtered::<&VoxelPos, With<Id<Structure>>>();

        for &voxel_pos in structure_query.iter(&app.world) {
            let terrain_height = map_geometry.get_height(voxel_pos.hex).unwrap();
            assert!(voxel_pos.height > terrain_height);
        }
    }

    #[test]
    fn structures_exist() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(Startup, (generate_terrain, generate_structures).chain());

        app.update();

        let map_geometry = app.world.resource::<MapGeometry>().clone();
        let mut structure_query = app
            .world
            .query_filtered::<(Entity, &VoxelPos), With<Id<Structure>>>();

        for (queried_entity, &voxel_pos) in structure_query.iter(&app.world) {
            let cached_structure_entity = map_geometry.get_structure(voxel_pos).unwrap();
            assert_eq!(queried_entity, cached_structure_entity);
        }
    }

    #[test]
    fn terrain_exists() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.add_systems(Startup, generate_terrain);
        app.insert_resource(GlobalRng::new(0));

        app.update();

        let map_geometry = app.world.resource::<MapGeometry>().clone();
        let mut terrain_query = app.world.query::<&Id<Terrain>>();

        for &hex in map_geometry.all_hexes() {
            let cached_terrain_entity = map_geometry.get_terrain(hex).unwrap();
            terrain_query
                .get(&app.world, cached_terrain_entity)
                .unwrap();
        }
    }

    #[test]
    fn can_generate_landmarks() {
        let mut app = App::new();
        app.add_plugin(DummyManifestPlugin);
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(Startup, (generate_terrain, generate_landmarks).chain());

        app.update();
    }

    #[test]
    fn can_generate_water() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.insert_resource(WaterConfig::IN_GAME);
        app.insert_resource(GlobalRng::new(0));
        app.add_systems(Startup, (generate_terrain, initialize_water_table).chain());

        app.update();
    }

    #[test]
    fn can_generate_world() {
        let mut app = App::new();
        app.add_plugin(GenerationPlugin {
            config: GenerationConfig::testing(),
        })
        .add_plugin(DummyManifestPlugin);
        app.insert_resource(GlobalRng::new(0));
        app.insert_resource(WaterConfig::IN_GAME);

        app.update();

        let mut unit_query = app.world.query::<&Id<Unit>>();
        assert!(
            unit_query.iter(&app.world).next().is_some(),
            "No units generated"
        );

        let mut structure_query = app.world.query::<&Id<Structure>>();
        assert!(
            structure_query.iter(&app.world).next().is_some(),
            "No structures generated"
        );
    }
}
