//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::OrganismPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::geometry::sync_rotation_to_facing;
use crate::structures::StructuresPlugin;
use bevy::app::{App, Plugin};
use bevy::log::info;

pub mod generation;
pub mod geometry;

/// All of the code needed to make the simulation run
pub struct SimulationPlugin {
    /// Configuration settings for world generation, these will be passed to [`GenerationPlugin`]
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
            .add_plugin(OrganismPlugin);
    }
}
