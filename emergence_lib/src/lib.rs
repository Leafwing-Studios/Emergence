//! Provides plugins needed by the Emergence game.
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
// Often exceeded by queries
#![allow(clippy::type_complexity)]

use bevy::prelude::In;

pub mod asset_management;
pub mod curves;
pub mod enum_iter;
pub mod graphics;
pub mod items;
pub mod organisms;
pub mod player_interaction;
pub mod signals;
pub mod simulation;
pub mod structures;
pub mod terrain;
pub mod ui;
pub mod units;

/// Various app configurations, used for testing.
///
/// Importing between files shared in the `tests` directory appears to be broken with this workspace config?
/// Followed directions from <https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html>
pub mod testing {
    use crate::simulation::generation::GenerationConfig;
    use crate::simulation::SimulationPlugin;
    use bevy::prelude::*;

    /// Just [`MinimalPlugins`].
    pub fn minimal_app() -> App {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);

        app
    }

    /// Just the game logic and simulation
    pub fn simulation_app(gen_config: GenerationConfig) -> App {
        let mut app = minimal_app();
        app.add_plugin(SimulationPlugin { gen_config });
        app
    }

    /// Test users interacting with the app
    pub fn interaction_app(gen_config: GenerationConfig) -> App {
        let mut app = simulation_app(gen_config);
        app.add_plugin(bevy::input::InputPlugin)
            .add_plugin(crate::player_interaction::InteractionPlugin);
        app
    }
}

/// A very lazy system pipe adaptor for handling errors
// TODO: replace me with Bevy 0.10 equivalent.
pub(crate) fn ignore_errors<T>(In(_input): In<T>) {}
