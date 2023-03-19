//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::geometry::sync_rotation_to_facing;
use crate::simulation::time::TemporalPlugin;
use crate::structures::StructuresPlugin;
use crate::terrain::TerrainPlugin;
use crate::units::UnitsPlugin;
use bevy::app::{App, Plugin};
use bevy::log::info;

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
