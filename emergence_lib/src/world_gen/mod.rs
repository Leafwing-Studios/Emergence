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

use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::log::info;
use bevy::prelude::IntoSystemAppConfigs;
use bevy::utils::HashMap;

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
        app.insert_resource(self.config.clone()).add_systems(
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
                .in_schedule(OnEnter(AssetState::FullyLoaded)),
        );
    }
}

/// Controls world generation strategy
#[derive(Resource, Debug, Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub(super) map_radius: u32,
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

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        let mut terrain_weights: HashMap<Id<Terrain>, f32> = HashMap::new();
        // FIXME: load from file somehow
        terrain_weights.insert(Id::from_name("loam".to_string()), 1.0);
        terrain_weights.insert(Id::from_name("muddy".to_string()), 0.3);
        terrain_weights.insert(Id::from_name("rocky".to_string()), 0.2);

        let mut landmark_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        landmark_chances.insert(Id::from_name("spring".to_string()), 5e-4);

        let mut unit_chances: HashMap<Id<Unit>, f32> = HashMap::new();
        unit_chances.insert(Id::from_name("ant".to_string()), 1e-2);

        let mut structure_chances: HashMap<Id<Structure>, f32> = HashMap::new();
        structure_chances.insert(Id::from_name("ant_hive".to_string()), 1e-3);
        structure_chances.insert(Id::from_name("acacia".to_string()), 2e-2);
        structure_chances.insert(Id::from_name("leuco".to_string()), 1e-2);

        GenerationConfig {
            map_radius: 60,
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
                seed: 3.0,
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
