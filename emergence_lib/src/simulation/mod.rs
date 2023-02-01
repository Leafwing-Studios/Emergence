//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::map::MapPositions;
use crate::structures::StructuresPlugin;
use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::log::info;
use bevy::prelude::{Commands, Query, Res, ResMut, With};

use self::map::TilePos;

pub mod generation;
pub mod map;

/// All of the code needed to make the simulation run
pub struct SimulationPlugin {
    /// Configuration settings for world generation, these will be passed to [`GenerationPlugin`]
    pub gen_config: GenerationConfig,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building simulation plugin...");
        app.add_plugin(GenerationPlugin {
            config: self.gen_config.clone(),
        })
        .add_plugin(StructuresPlugin)
        .add_plugin(OrganismPlugin)
        .add_plugin(SignalsPlugin)
    }
}
