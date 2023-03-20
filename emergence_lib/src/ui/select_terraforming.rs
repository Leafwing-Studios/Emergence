//! Quickly select which terraforming option to use.

use crate::{
    asset_management::manifest::TerrainManifest,
    player_interaction::{
        terraform::{SelectedTerraforming, TerraformingChoice},
        PlayerAction,
    },
};

use itertools::Itertools;

use bevy::prelude::*;

use super::wheel_menu::{
    select_hex, spawn_hex_menu, AvailableChoices, Choice, HexMenu, HexMenuArrangement,
    HexMenuElement, HexMenuError,
};

/// Logic used to let users select the terraforming option to use.
pub(super) struct SelectTerraformingPlugin;

impl Plugin for SelectTerraformingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableChoices<TerraformingChoice>>()
            .add_systems(
                (
                    update_terraforming_choices,
                    spawn_hex_menu::<TerraformingChoice>,
                )
                    .chain(),
            )
            .add_system(select_hex.pipe(handle_selection));
    }
}

impl Choice for TerraformingChoice {
    const ACTIVATION: PlayerAction = PlayerAction::Terraform;
}

/// Update the set of choices available to build whenever the terrain manifest is updated
fn update_terraforming_choices(
    mut available_choices: ResMut<AvailableChoices<TerraformingChoice>>,
    terrain_manifest: Res<TerrainManifest>,
) {
    if terrain_manifest.is_changed() {
        available_choices.choices = vec![TerraformingChoice::Raise, TerraformingChoice::Lower];

        // Sort to ensure a stable ordering
        let terrain_choices = terrain_manifest
            .variants()
            .into_iter()
            .sorted()
            .map(|terrain_id| TerraformingChoice::Change(terrain_id));
        available_choices.choices.extend(terrain_choices);
    }
}

/// Set the selected terraforming choice based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuElement<TerraformingChoice>, HexMenuError>>,
    menu_query: Query<Entity, With<HexMenu>>,
    mut terraforming: ResMut<SelectedTerraforming>,
    commands: Commands,
) {
    /// Clean up the menu when we are done with it
    fn cleanup(mut commands: Commands, menu_query: Query<Entity, With<HexMenu>>) {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.remove_resource::<HexMenuArrangement<TerraformingChoice>>();
    }

    match result {
        Ok(element) => terraforming.current_selection = Some(element.data().clone()),
        Err(HexMenuError::NoSelection { complete }) => {
            if complete {
                terraforming.current_selection = None;
                cleanup(commands, menu_query);
            }
        }
        Err(HexMenuError::NoMenu) => (),
    }
}
