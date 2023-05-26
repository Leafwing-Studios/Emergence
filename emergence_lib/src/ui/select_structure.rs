//! Quickly select which structure to place.

use crate::{
    asset_management::{manifest::Id, AssetState},
    geometry::Facing,
    graphics::palette::ui::{MENU_HIGHLIGHT_COLOR, MENU_NEUTRAL_COLOR},
    player_interaction::{
        clipboard::{ClipboardData, Tool},
        PlayerAction,
    },
    structures::structure_manifest::{Structure, StructureManifest},
};

use itertools::Itertools;

use bevy::prelude::*;

use super::wheel_menu::{
    select_hex, spawn_hex_menu, AvailableChoices, Choice, HexMenu, HexMenuArrangement,
    HexMenuElement, HexMenuError,
};

/// Logic used to let users select the structure to build.
pub(super) struct SelectStructurePlugin;

impl Plugin for SelectStructurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableChoices<Id<Structure>>>()
            .add_systems(
                (update_structure_choices, spawn_hex_menu::<Id<Structure>>)
                    .chain()
                    .distributive_run_if(in_state(AssetState::FullyLoaded)),
            )
            .add_system(
                select_hex
                    .pipe(handle_selection)
                    .run_if(resource_exists::<HexMenuArrangement<Id<Structure>>>()),
            );
    }
}

impl Choice for Id<Structure> {
    const ACTIVATION: PlayerAction = PlayerAction::SelectStructure;
}

/// Update the set of choices available to build whenever the structure manifest is updated
fn update_structure_choices(
    mut available_choices: ResMut<AvailableChoices<Id<Structure>>>,
    structure_manifest: Res<StructureManifest>,
) {
    if structure_manifest.is_changed() {
        // Sort to ensure a stable ordering
        available_choices.choices = structure_manifest
            .prototypes()
            .into_iter()
            .sorted()
            .collect();
    }
}

/// Set the selected structure based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuElement<Id<Structure>>, HexMenuError>>,
    mut tool: ResMut<Tool>,
    menu_query: Query<Entity, With<HexMenu>>,
    mut background_query: Query<&mut BackgroundColor, With<HexMenu>>,
    structure_manifest: Res<StructureManifest>,
    commands: Commands,
    arrangement: Res<HexMenuArrangement<Id<Structure>>>,
) {
    /// Clean up the menu when we are done with it
    fn cleanup(mut commands: Commands, menu_query: Query<Entity, With<HexMenu>>) {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.remove_resource::<HexMenuArrangement<Id<Structure>>>();
    }

    match result {
        Ok(element) => {
            if element.is_complete() {
                let structure_data = ClipboardData {
                    structure_id: *element.data(),
                    facing: Facing::default(),
                    active_recipe: structure_manifest
                        .get(*element.data())
                        .starting_recipe()
                        .clone(),
                };

                tool.set_to_structure(Some(structure_data));
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
            tool.set_to_structure(None);

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
