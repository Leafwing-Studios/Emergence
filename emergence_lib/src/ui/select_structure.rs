//! Quickly select which structure to place.

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest},
    player_interaction::clipboard::{Clipboard, ClipboardData},
    simulation::geometry::Facing,
};

use bevy::prelude::*;

use super::wheel_menu::{
    select_hex, spawn_hex_menu, HexMenu, HexMenuArrangement, HexMenuData, HexMenuError,
};

/// Hex menu and selection modifying logic.
pub(super) struct SelectStructurePlugin;

impl Plugin for SelectStructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_hex_menu)
            .add_system(select_hex.pipe(handle_selection));
    }
}

/// Set the selected structure based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuData, HexMenuError>>,
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

        commands.remove_resource::<HexMenuArrangement>();
    }

    match result {
        Ok(data) => {
            if data.is_complete() {
                let structure_data = ClipboardData {
                    structure_id: data.structure_id(),
                    facing: Facing::default(),
                    active_recipe: structure_manifest
                        .get(data.structure_id())
                        .starting_recipe()
                        .clone(),
                };

                clipboard.set(Some(structure_data));
                cleanup(commands, menu_query);
            } else {
                for (icon_entity, &structure_id, mut icon_color) in icon_query.iter_mut() {
                    if icon_entity == data.icon_entity() {
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
