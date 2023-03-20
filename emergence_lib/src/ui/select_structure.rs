//! Quickly select which structure to place.

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest},
    player_interaction::clipboard::{Clipboard, ClipboardData},
    simulation::geometry::Facing,
};

use bevy::prelude::*;

use super::wheel_menu::{
    select_hex, spawn_hex_menu, AvailableChoices, HexMenu, HexMenuArrangement, HexMenuElement,
    HexMenuError,
};

/// Hex menu and selection modifying logic.
pub(super) struct SelectStructurePlugin;

impl Plugin for SelectStructurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableChoices<Id<Structure>>>()
            .add_system(spawn_hex_menu::<Id<Structure>>)
            .add_system(select_hex.pipe(handle_selection));
    }
}

impl FromWorld for AvailableChoices<Id<Structure>> {
    fn from_world(world: &mut World) -> Self {
        todo!()
    }
}

/// Set the selected structure based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuElement<Id<Structure>>, HexMenuError>>,
    mut clipboard: ResMut<Clipboard>,
    menu_query: Query<Entity, With<HexMenu>>,
    mut icon_query: Query<(Entity, &Id<Structure>, &mut BackgroundColor), With<HexMenu>>,
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
            } else {
                for (icon_entity, &structure_id, mut icon_color) in icon_query.iter_mut() {
                    if icon_entity == element.icon_entity() {
                        *icon_color = BackgroundColor(Color::ANTIQUE_WHITE);
                    } else {
                        *icon_color = BackgroundColor(structure_manifest.get(structure_id).color);
                    }
                }
            }
        }
        Err(HexMenuError::NoSelection { complete }) => {
            clipboard.set(None);

            if complete {
                cleanup(commands, menu_query);
            } else {
                for (_icon_entity, &structure_id, mut icon_color) in icon_query.iter_mut() {
                    *icon_color = BackgroundColor(structure_manifest.get(structure_id).color);
                }
            }
        }
        Err(HexMenuError::NoMenu) => (),
    }
}
