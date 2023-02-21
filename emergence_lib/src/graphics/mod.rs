//! Rendering and animation logic.

use bevy::prelude::*;

use crate::{asset_management::AssetState, player_interaction::InteractionSystem};

use self::lighting::LightingPlugin;

mod lighting;
mod structures;
mod terrain;
mod units;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LightingPlugin)
            .add_system_set(
                SystemSet::on_update(AssetState::Ready)
                    .with_system(terrain::populate_terrain)
                    .with_system(units::populate_units)
                    .with_system(units::display_held_item)
                    // We need to avoid attempting to insert bundles into entities that no longer exist
                    .with_system(
                        structures::populate_structures.before(InteractionSystem::ManagePreviews),
                    ),
            )
            .add_system_to_stage(CoreStage::PostUpdate, structures::change_structure_material)
            .add_system(
                terrain::display_tile_interactions
                    .after(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            );
    }
}
