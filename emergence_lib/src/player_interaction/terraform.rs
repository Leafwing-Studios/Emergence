//! Tools to alter the terrain type and height.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{Id, Terrain},
    simulation::geometry::Height,
};

use super::{selection::CurrentSelection, InteractionSystem};

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

/// Flags terrain to be terraformed based on player selection.
fn mark_for_terraforming(current_selection: Res<CurrentSelection>, mut commands: Commands) {}

/// Changes the terrain to match the [`Terraform`] component
fn apply_terraforming(
    query: Query<(&mut Id<Terrain>, &mut Height, &Terraform)>,
    mut commands: Commands,
) {
}
