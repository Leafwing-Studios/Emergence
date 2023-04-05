//! Tools and systems for constructing structures and terraforming the world.

use bevy::prelude::*;

pub(crate) mod demolition;
pub(crate) mod ghosts;
pub(crate) mod terraform;
pub(crate) mod zoning;

/// Systems and resources for constructing structures and terraforming the world.
pub(crate) struct ConstructionPlugin;

impl Plugin for ConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ghosts::GhostPlugin)
            .add_plugin(terraform::TerraformingPlugin)
            .add_plugin(zoning::ZoningPlugin);
    }
}
