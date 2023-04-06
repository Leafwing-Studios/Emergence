//! Tools to alter the terrain type and height.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    crafting::components::CraftingState,
    player_interaction::InteractionSystem,
    simulation::{
        geometry::{Height, MapGeometry, TilePos},
        SimulationSet,
    },
    terrain::{
        commands::TerrainCommandsExt,
        terrain_assets::TerrainHandles,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

use super::{demolition::MarkedForDemolition, ghosts::Ghost, zoning::Zoning};

/// Systems that handle terraforming.
pub(super) struct TerraformingPlugin;

impl Plugin for TerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                spawn_terraforming_ghosts,
                apply_terraforming_when_ghosts_complete,
            )
                .in_set(InteractionSystem::ApplyTerraforming)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// An option presented to players for how to terraform the world.
///
/// These are generally higher level than the actual [`TerraformingAction`]s,
/// which represent the actual changes to the terrain that can be performed by units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingTool {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided [`Id<Terrain>`].
    Change(Id<Terrain>),
}

/// When `Zoning` is set, this is added  as a component added to terrain ghosts, causing them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TerraformingAction {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Set the desired terrain material of this tile
    Change(Id<Terrain>),
}

impl TerraformingAction {
    /// The pretty formatted name of this action.
    pub(crate) fn display(&self, terrain_manifest: &TerrainManifest) -> String {
        match self {
            Self::Raise => "Raise".to_string(),
            Self::Lower => "Lower".to_string(),
            Self::Change(terrain_id) => terrain_manifest.name(*terrain_id).to_string(),
        }
    }
}

impl From<TerraformingTool> for TerraformingAction {
    fn from(choice: TerraformingTool) -> Self {
        match choice {
            TerraformingTool::Raise => Self::Raise,
            TerraformingTool::Lower => Self::Lower,
            TerraformingTool::Change(terrain) => Self::Change(terrain),
        }
    }
}

/// Changes the terrain to match the [`MarkedForTerraforming`] component
fn spawn_terraforming_ghosts(
    mut terrain_query: Query<(&TilePos, Ref<Zoning>, &Id<Terrain>)>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    for (&tile_pos, zoning, current_terrain_id) in terrain_query.iter_mut() {
        if zoning.is_changed() {
            if let Zoning::Terraform(terraforming_action) = *zoning {
                // We neeed to use the model for the terrain we're changing to, not the current one
                let terrain_id = match terraforming_action {
                    TerraformingAction::Change(terrain_id) => terrain_id,
                    _ => *current_terrain_id,
                };

                commands.spawn_ghost_terrain(tile_pos, terrain_id, terraforming_action);

                // Mark any structures that are here as needing to be demolished
                // Terraforming can't be done with roots growing into stuff!
                if let Some(structure_entity) = map_geometry.get_structure(tile_pos) {
                    commands
                        .entity(structure_entity)
                        .insert(MarkedForDemolition);
                }
            }
        }
    }
}

/// Changes the terrain to match the [`MarkedForTerraforming`] component
fn apply_terraforming_when_ghosts_complete(
    mut terrain_query: Query<(
        &mut Zoning,
        &mut Id<Terrain>,
        &mut Height,
        &mut Handle<Scene>,
    )>,
    ghost_query: Query<(Ref<CraftingState>, &TilePos, &TerraformingAction), With<Ghost>>,
    terrain_handles: Res<TerrainHandles>,
    mut map_geometry: ResMut<MapGeometry>,
    mut commands: Commands,
) {
    for (crafting_state, &tile_pos, terraforming_action) in ghost_query.iter() {
        // FIXME: ensure that terraforming only progresses when no structures are present
        if matches!(*crafting_state, CraftingState::RecipeComplete) {
            commands.despawn_ghost_terrain(tile_pos)
        }

        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let (mut zoning, mut terrain, mut height, mut scene_handle) =
            terrain_query.get_mut(terrain_entity).unwrap();

        match terraforming_action {
            TerraformingAction::Raise => *height += Height(1),
            TerraformingAction::Lower => *height -= Height(1),
            TerraformingAction::Change(terrain_id) => {
                *terrain = *terrain_id;
                *scene_handle = terrain_handles.scenes.get(terrain_id).unwrap().clone_weak();
            }
        };

        map_geometry.update_height(tile_pos, *height);
        *zoning = Zoning::None;
    }
}
