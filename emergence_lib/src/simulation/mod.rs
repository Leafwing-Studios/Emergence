//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::asset_management::AssetState;
use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::geometry::sync_rotation_to_facing;
use crate::simulation::time::TemporalPlugin;
use crate::structures::StructuresPlugin;
use crate::terrain::TerrainPlugin;
use crate::units::UnitsPlugin;
use bevy::prelude::*;

pub mod generation;
pub mod geometry;
pub mod time;

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
            .add_plugin(StructuresPlugin)
            .add_plugin(TerrainPlugin)
            .add_plugin(OrganismPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(SignalsPlugin)
            .add_plugin(TemporalPlugin);
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
