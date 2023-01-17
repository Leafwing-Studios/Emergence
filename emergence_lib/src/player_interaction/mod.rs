//! Tools for the player to interact with the world

use bevy::prelude::{App, Plugin};

pub mod camera;
pub mod cursor;
pub mod hive_mind;
pub mod organism_details;

/// All of the code needed for users to interact with the simulation.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera::CameraPlugin)
            .add_plugin(cursor::CursorTilePosPlugin)
            .add_plugin(organism_details::DetailsPlugin)
            .add_plugin(hive_mind::HiveMindPlugin);

        #[cfg(feature = "debug_tools")]
        app.add_plugin(debug_tools::DebugToolsPlugin);
    }
}
