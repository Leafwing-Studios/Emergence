//! Tools to alter the terrain type and height.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::{
        manifest::{Id, Terrain},
        terrain::TerrainHandles,
    },
    simulation::geometry::{Height, MapGeometry, TilePos},
    structures::commands::StructureCommandsExt,
};

use super::{cursor::CursorPos, selection::CurrentSelection, InteractionSystem, PlayerAction};

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTerraforming>().add_systems(
            (mark_for_terraforming, apply_terraforming)
                .in_set(InteractionSystem::ApplyTerraforming),
        );
    }
}

/// An option for how to terraform the world.
///
/// Also used as a component added to terrain entities that marks them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingChoice {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided Id<Terrain>.
    Change(Id<Terrain>),
}

/// Currently selected terraforming settings
#[derive(Resource, Default)]
pub(crate) struct SelectedTerraforming {
    pub(crate) current_selection: Option<TerraformingChoice>,
}

/// Flags terrain to be terraformed based on player selection + actions.
fn mark_for_terraforming(
    current_selection: Res<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    map_geometry: Res<MapGeometry>,
    actions: Res<ActionState<PlayerAction>>,
    terraforming: Res<SelectedTerraforming>,
    mut commands: Commands,
) {
    if actions.just_pressed(PlayerAction::Zone) {
        if let Some(terraforming_choice) = terraforming.current_selection {
            let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
            let relevant_terrain_entities = relevant_tiles.entities(&map_geometry);

            for terrain_entity in relevant_terrain_entities {
                // TODO: this should be player-configurable
                commands.entity(terrain_entity).insert(terraforming_choice);
            }
        }
    }
}

/// Changes the terrain to match the [`Terraform`] component
fn apply_terraforming(
    mut query: Query<(
        Entity,
        &TerraformingChoice,
        &TilePos,
        &mut Id<Terrain>,
        &mut Height,
        &mut Handle<Scene>,
    )>,
    terrain_handles: Res<TerrainHandles>,
    mut commands: Commands,
) {
    // TODO: this should take work.
    for (entity, terraform, tile_pos, mut terrain, mut height, mut scene_handle) in query.iter_mut()
    {
        match terraform {
            TerraformingChoice::Raise => *height += Height(1),
            TerraformingChoice::Lower => *height -= Height(1),
            TerraformingChoice::Change(target_terrain_type) => {
                *terrain = *target_terrain_type;
                *scene_handle = terrain_handles
                    .scenes
                    .get(target_terrain_type)
                    .unwrap()
                    .clone_weak();
            }
        }

        // Don't keep the component around
        commands.entity(entity).remove::<TerraformingChoice>();

        // Despawn any structures here; terraforming can't be done with roots growing into stuff!
        commands.despawn_structure(*tile_pos);
    }
}
