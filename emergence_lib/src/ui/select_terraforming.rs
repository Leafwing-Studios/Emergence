//! Quickly select which terraforming option to use.

use crate::{
    graphics::palette::ui::{MENU_HIGHLIGHT_COLOR, MENU_NEUTRAL_COLOR},
    player_interaction::{clipboard::Clipboard, terraform::TerraformingChoice, PlayerAction},
    terrain::terrain_manifest::TerrainManifest,
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
            .add_system(
                select_hex
                    .pipe(handle_selection)
                    .run_if(resource_exists::<HexMenuArrangement<TerraformingChoice>>()),
            );
    }
}

impl Choice for TerraformingChoice {
    const ACTIVATION: PlayerAction = PlayerAction::SelectTerraform;
}

/// Update the set of choices available to build whenever the terrain manifest is updated
fn update_terraforming_choices(
    mut available_choices: ResMut<AvailableChoices<TerraformingChoice>>,
    terrain_manifest: Res<TerrainManifest>,
) {
    if terrain_manifest.is_changed() {
        available_choices.choices = vec![TerraformingChoice::Raise, TerraformingChoice::Lower];

        // Sort to ensure a stable ordering
        // The lint here is just wrong
        #[allow(clippy::redundant_closure)]
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
    mut background_query: Query<&mut BackgroundColor, With<HexMenu>>,
    menu_query: Query<Entity, With<HexMenu>>,
    mut clipboard: ResMut<Clipboard>,
    commands: Commands,
    arrangement: Res<HexMenuArrangement<TerraformingChoice>>,
) {
    /// Clean up the menu when we are done with it
    fn cleanup(mut commands: Commands, menu_query: Query<Entity, With<HexMenu>>) {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.remove_resource::<HexMenuArrangement<TerraformingChoice>>();
    }

    match result {
        Ok(element) => {
            if element.is_complete() {
                *clipboard = Clipboard::Terraform(*element.data());
                cleanup(commands, menu_query);
            } else {
                for (&background_hex, &background_entity) in arrangement.background_map() {
                    if let Ok(mut background_color) = background_query.get_mut(background_entity) {
                        *background_color = if background_hex == element.hex() {
                            BackgroundColor(MENU_HIGHLIGHT_COLOR)
                        } else {
                            BackgroundColor(MENU_NEUTRAL_COLOR)
                        }
                    }
                }
            }
        }
        Err(HexMenuError::NoSelection { complete }) => {
            *clipboard = Clipboard::Empty;
            if complete {
                cleanup(commands, menu_query);
            } else {
                for &background_entity in arrangement.background_map().values() {
                    if let Ok(mut background_color) = background_query.get_mut(background_entity) {
                        *background_color = BackgroundColor(MENU_NEUTRAL_COLOR)
                    }
                }
            }
        }
        Err(HexMenuError::NoMenu) => (),
    }
}
