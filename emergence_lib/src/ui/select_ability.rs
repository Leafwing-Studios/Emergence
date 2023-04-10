//! Quickly select which structure to place.

use crate::{
    asset_management::{manifest::Id, AssetState},
    graphics::palette::ui::{MENU_HIGHLIGHT_COLOR, MENU_NEUTRAL_COLOR},
    player_interaction::{abilities::IntentAbility, clipboard::Tool, PlayerAction},
    structures::structure_manifest::Structure,
};

use bevy::prelude::*;

use super::wheel_menu::{
    select_hex, spawn_hex_menu, AvailableChoices, Choice, HexMenu, HexMenuArrangement,
    HexMenuElement, HexMenuError,
};

/// Logic used to let users select the ability to use.
pub(super) struct SelectAbilityPlugin;

impl Plugin for SelectAbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableChoices<IntentAbility>>()
            .add_system(spawn_hex_menu::<IntentAbility>.run_if(in_state(AssetState::FullyLoaded)))
            .add_system(
                select_hex
                    .pipe(handle_selection)
                    .run_if(resource_exists::<HexMenuArrangement<IntentAbility>>()),
            );
    }
}

impl Choice for IntentAbility {
    const ACTIVATION: PlayerAction = PlayerAction::SelectAbility;
}

/// Set the selected ability based on the results of the hex menu.
fn handle_selection(
    In(result): In<Result<HexMenuElement<IntentAbility>, HexMenuError>>,
    mut tool: ResMut<Tool>,
    menu_query: Query<Entity, With<HexMenu>>,
    mut background_query: Query<&mut BackgroundColor, With<HexMenu>>,
    commands: Commands,
    arrangement: Res<HexMenuArrangement<IntentAbility>>,
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
                *tool = Tool::Ability(element.data().clone());
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
            *tool = Tool::None;

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
