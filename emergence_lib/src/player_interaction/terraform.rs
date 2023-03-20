//! Tools to alter the terrain type and height.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::manifest::{Id, Terrain},
    simulation::geometry::{Height, MapGeometry},
};

use super::{cursor::CursorPos, selection::CurrentSelection, InteractionSystem, PlayerAction};

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (mark_for_terraforming, apply_terraforming)
                .in_set(InteractionSystem::ApplyTerraforming),
        );
    }
}

/// A component added to terrain entities that marks them to be manipulated by units.
#[derive(Component)]
struct Terraform {
    /// The terrain type to change this into.
    target_terrain_type: Id<Terrain>,
    /// The desired height of the terrain.
    target_height: Height,
}

/// Flags terrain to be terraformed based on player selection + actions.
fn mark_for_terraforming(
    current_selection: Res<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    map_geometry: Res<MapGeometry>,
    actions: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
) {
    if actions.just_pressed(PlayerAction::Terraform) {
        let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
        let relevant_terrain_entities = relevant_tiles.entities(&map_geometry);

        for terrain_entity in relevant_terrain_entities {
            // TODO: this should be player-configurable
            commands.entity(terrain_entity).insert(Terraform {
                target_terrain_type: Id::from_name("rocky"),
                target_height: Height::MIN,
            });
        }
    }
}

/// Changes the terrain to match the [`Terraform`] component
fn apply_terraforming(
    mut query: Query<(Entity, &mut Id<Terrain>, &mut Height, &Terraform)>,
    mut commands: Commands,
) {
    // FIXME: we should be careful not to let this happen until all non-matching structures are removed.
    // FIXME: terrain visuals are not updated
    // TODO: this should take work.
    for (entity, mut terrain, mut height, terraform) in query.iter_mut() {
        *terrain = terraform.target_terrain_type;
        *height = terraform.target_height;

        // Don't keep the component around
        commands.entity(entity).remove::<Terraform>();
    }
}
