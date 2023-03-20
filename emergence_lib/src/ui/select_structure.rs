//! Quickly select which structure to place.

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest},
    player_interaction::{
        clipboard::{Clipboard, ClipboardData},
        PlayerAction,
    },
    simulation::geometry::Facing,
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
            .add_systems((update_structure_choices, spawn_hex_menu::<Id<Structure>>).chain())
            .add_system(select_hex.pipe(handle_selection));
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
        available_choices.choices = structure_manifest.variants().into_iter().sorted().collect();
    }
}

/// Set the selected structure based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuElement<Id<Structure>>, HexMenuError>>,
    mut clipboard: ResMut<Clipboard>,
    menu_query: Query<Entity, With<HexMenu>>,
    structure_manifest: Res<StructureManifest>,
    commands: Commands,
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

                clipboard.set(Some(structure_data));
                cleanup(commands, menu_query);
            }
        }
        Err(HexMenuError::NoSelection { complete }) => {
            clipboard.set(None);

            if complete {
                cleanup(commands, menu_query);
            }
        }
        Err(HexMenuError::NoMenu) => (),
    }
}
