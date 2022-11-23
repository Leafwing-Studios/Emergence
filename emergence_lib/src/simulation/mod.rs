//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::structures::StructuresPlugin;
use crate::organisms::units::UnitsPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::GenerationPlugin;
use crate::simulation::map::MapPositions;
use crate::simulation::pathfinding::{Impassable, PassableFilters};
use bevy::app::{App, CoreStage, Plugin};
use bevy::log::info;
use bevy::prelude::{Commands, Query, Res, With};
use bevy_ecs_tilemap::tiles::TilePos;

pub mod generation;
pub mod map;
pub mod pathfinding;

/// All of the code needed to make the simulation run
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building simulation plugin...");
        app.add_plugin(GenerationPlugin)
            .add_plugin(StructuresPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(SignalsPlugin)
            .add_system_to_stage(CoreStage::First, create_passable_filter);
    }
}

/// Create the [`PassableFilter`] resource
pub fn create_passable_filter(
    mut commands: Commands,
    impassable_query: Query<&TilePos, With<Impassable>>,
    map_positions: Res<MapPositions>,
) {
    commands.insert_resource(PassableFilters::from_impassable_query(
        &impassable_query,
        &map_positions,
    ));
}
