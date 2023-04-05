//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::asset_management::AssetState;
use crate::construction::ConstructionPlugin;
use crate::crafting::CraftingPlugin;
use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::geometry::{sync_rotation_to_facing, MapGeometry};
use crate::simulation::light::LightPlugin;
use crate::simulation::time::TemporalPlugin;
use crate::structures::StructuresPlugin;
use crate::terrain::TerrainPlugin;
use crate::units::UnitsPlugin;
use bevy::prelude::*;

pub mod generation;
pub mod geometry;
pub mod light;
pub mod time;

/// Sets up world geometry
pub struct GeometryPlugin {
    /// Configuration settings for world generation
    pub gen_config: GenerationConfig,
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapGeometry::new(self.gen_config.map_radius));
    }
}

/// All of the code needed to make the simulation run
pub struct SimulationPlugin {
    /// Configuration settings for world generation
    pub gen_config: GenerationConfig,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building simulation plugin...");
        app.add_system(sync_rotation_to_facing)
            .add_state::<PauseState>()
            .insert_resource(FixedTime::new_from_secs(1.0 / 30.))
            .edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
                schedule.configure_set(
                    SimulationSet
                        .run_if(in_state(PauseState::Playing))
                        .run_if(in_state(AssetState::Ready)),
                );
            })
            .add_plugin(GenerationPlugin {
                config: self.gen_config.clone(),
            })
            .add_plugin(CraftingPlugin)
            .add_plugin(ConstructionPlugin)
            .add_plugin(StructuresPlugin)
            .add_plugin(TerrainPlugin)
            .add_plugin(OrganismPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(SignalsPlugin)
            .add_plugin(TemporalPlugin)
            .add_plugin(LightPlugin);
    }
}

/// Controls whether or not the game is paused.
#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
enum PauseState {
    /// Game logic is running.
    #[default]
    Playing,
    /// Game logic is stopped.
    Paused,
}

/// Simulation systems.
///
/// These:
/// - are run in [`CoreSchedule::FixedUpdate`]
/// - only run in [`PauseState::Playing`]
/// - only run in [`AssetState::Ready`]
#[derive(SystemSet, PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct SimulationSet;
