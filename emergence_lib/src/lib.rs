//! Provides plugins needed by the Emergence game.
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]

use bevy::app::{App, Plugin};

pub mod camera;
pub mod cursor;
pub mod curves;
pub mod enum_iter;
pub mod hive_mind;
pub mod organisms;
pub mod signals;
pub mod terrain;
pub mod tiles;

/// All of the code needed to make the simulation run
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(hive_mind::HiveMindPlugin)
            .add_plugin(terrain::generation::GenerationPlugin)
            .add_plugin(organisms::structures::StructuresPlugin)
            .add_plugin(organisms::units::UnitsPlugin)
            .add_plugin(signals::SignalsPlugin);
    }
}

/// All of the code needed for users to interact with the simulation.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera::CameraPlugin)
            .add_plugin(cursor::CursorTilePosPlugin);
    }
}

/// All of the code needed to draw things on screen.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin);
    }
}
