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
pub mod graphics;
pub mod hive_mind;
pub mod interactable;
pub mod organisms;
pub mod signals;
pub mod simulation;
pub mod terrain;

/// All of the code needed to make the simulation run
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(simulation::generation::GenerationPlugin)
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
            .add_plugin(cursor::CursorTilePosPlugin)
            .add_plugin(hive_mind::HiveMindPlugin);
    }
}

/// Various app configurations, used for testing.
///
/// Importing between files shared in the `tests` directory appears to be broken with this workspace config?
/// Followed directions from <https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html>
pub mod testing {
    use bevy::prelude::*;

    /// Just [`MinimalPlugins`].
    pub fn minimal_app() -> App {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);

        app
    }

    /// The [`bevy`] plugins needed to make simulation work
    pub fn bevy_app() -> App {
        let mut app = minimal_app();
        app.insert_resource(bevy::render::settings::WgpuSettings {
            backends: None,
            ..default()
        });

        app.add_plugin(bevy::asset::AssetPlugin)
            .add_plugin(bevy::window::WindowPlugin)
            .add_plugin(bevy::render::RenderPlugin);
        app
    }

    /// Just the game logic and simulation
    pub fn simulation_app() -> App {
        let mut app = bevy_app();
        app.add_plugin(super::SimulationPlugin);
        app
    }

    /// Test users interacting with the app
    pub fn interaction_app() -> App {
        let mut app = simulation_app();
        app.add_plugin(bevy::input::InputPlugin)
            .add_plugin(super::InteractionPlugin);
        app
    }
}
