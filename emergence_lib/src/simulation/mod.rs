//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::organisms::structures::StructuresPlugin;
use crate::organisms::units::UnitsPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::generation::GenerationPlugin;
use crate::simulation::map::MapPositions;
use crate::simulation::pathfinding::{Impassable, PassabilityCache};
use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::log::info;
use bevy::prelude::{Commands, Query, Res, ResMut, With};
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
            .add_startup_system_to_stage(StartupStage::PostStartup, initialize_passable_filter)
            .add_system_to_stage(CoreStage::PreUpdate, update_passable_filter);
    }
}

/// Create the [`PassabilityCache`] resource
pub fn initialize_passable_filter(mut commands: Commands, map_positions: Res<MapPositions>) {
    commands.insert_resource(PassabilityCache::new(&map_positions));
}

// /// Create the [`PassableFilters`] resource
// pub fn update_passable_filter(
//     newly_impassable: Query<&TilePos, (With<Impassable>, Changed<Impassable>)>,
//     newly_passable: Query<&TilePos, (Without<Impassable>, Changed<Impassable>)>,
//     mut passable_filters: ResMut<PassableFilters>,
// ) {
//     passable_filters.update_from_changed_passable_queries(&newly_impassable, &newly_passable);
// }

/// Update the [`PassabilityCache`] resource
pub fn update_passable_filter(
    impassable: Query<&TilePos, With<Impassable>>,
    mut passable_filters: ResMut<PassabilityCache>,
) {
    passable_filters.update_from_impassable_query(&impassable)
}
