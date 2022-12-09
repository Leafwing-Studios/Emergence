//! Creates the UI from all modules.
//!
use bevy::prelude::Plugin;

pub mod pheromone;

/// Struct to build the UI plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}
