//! Generating starting terrain and organisms
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetState;
use crate::structures::structure_manifest::Structure;
use crate::terrain::terrain_manifest::Terrain;
use crate::units::unit_manifest::Unit;
use crate::utils::noise::SimplexSettings;
use crate::world_gen::organism_generation::{generate_organisms, randomize_starting_organisms};
use crate::world_gen::terrain_generation::{
    generate_landmarks, generate_terrain, initialize_water_table,
};

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_framepace::{FramepaceSettings, Limiter};

mod organism_generation;
mod terrain_generation;

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
                (
                    generate_terrain,
                    apply_system_buffers,
                    generate_landmarks,
                    initialize_water_table,
                    apply_system_buffers,
                    generate_organisms,
                    apply_system_buffers,
                    randomize_starting_organisms,
                )
                    .chain()
                    .in_schedule(OnEnter(WorldGenState::Generating)),
            )
            .add_system(
                WorldGenState::manage_state
                    .in_base_set(CoreSet::PreUpdate)
                    .run_if(|world_gen_state: Res<State<WorldGenState>>| {
                        world_gen_state.0 != WorldGenState::Complete
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
        match world_gen_state.0 {
            WorldGenState::Waiting => {
                if let Some(frame_pace_settings) = maybe_frame_pace_settings.as_mut() {
                    // Don't limit the tick rate while generating the world
                    if !matches!(frame_pace_settings.limiter, Limiter::Off) {
                        frame_pace_settings.limiter = Limiter::Off;
                    }
                }

                if let Some(asset_state) = maybe_asset_state {
                    if asset_state.0 == AssetState::FullyLoaded {
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
            map_radius: 80,
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
                seed: 315.0,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 1.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
                seed: 100.0,
            },
        }
    }

    /// A tiny world gen config for testing.
    pub fn testing() -> Self {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("grassy".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("swampy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();

        let unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();

        let structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();

        GenerationConfig {
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
                seed: 315.0,
            },
            high_frequency_noise: SimplexSettings {
                frequency: 0.1,
                amplitude: 1.0,
                octaves: 2,
                lacunarity: 2.3,
                gain: 0.5,
                seed: 100.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_terrain() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.add_startup_system(generate_terrain);

        app.update();
    }

    #[test]
    fn can_generate_organisms() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.add_startup_systems((generate_terrain, generate_organisms).chain());

        app.update();
    }

    #[test]
    fn can_generate_landmarks() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.add_startup_systems((generate_terrain, generate_organisms).chain());

        app.update();
    }

    #[test]
    fn can_generate_water() {
        let mut app = App::new();
        app.insert_resource(GenerationConfig::testing());
        app.add_startup_systems((generate_terrain, initialize_water_table).chain());

        app.update();
    }

    #[test]
    fn can_generate_world() {
        let mut app = App::new();
        app.add_plugin(GenerationPlugin {
            config: GenerationConfig::testing(),
        });
        app.update();
    }
}
