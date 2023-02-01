//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::{GenerationConfig, GenerationPlugin};
use crate::simulation::map::MapPositions;
use crate::simulation::pathfinding::{Impassable, PassabilityCache};
use crate::structures::StructuresPlugin;
use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::log::info;
use bevy::prelude::{Commands, Query, Res, ResMut, With};

pub mod generation;
pub mod map;
pub mod pathfinding;

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
        .add_startup_system_to_stage(StartupStage::PostStartup, initialize_passable_filter)
        .add_system_to_stage(CoreStage::PreUpdate, update_passable_filter);
    }
}

/// Create the [`PassabilityCache`] resource
pub fn initialize_passable_filter(mut commands: Commands, map_positions: Res<MapPositions>) {
    commands.insert_resource(PassabilityCache::new(&map_positions));
}

/// Update the [`PassabilityCache`] resource
pub fn update_passable_filter(
    impassable: Query<&TilePos, With<Impassable>>,
    mut passable_filters: ResMut<PassabilityCache>,
) {
    passable_filters.update_from_impassable_query(&impassable)
}
