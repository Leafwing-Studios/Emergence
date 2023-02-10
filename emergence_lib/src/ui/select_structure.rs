//! Quickly select which structure to place.

use bevy::{prelude::*, utils::HashMap};
use hexx::Hex;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    player_interaction::{
        clipboard::{Clipboard, StructureData},
        cursor::CursorPos,
        PlayerAction,
    },
    structures::StructureId,
};

/// Hex menu and selection modifying logic.
pub(super) struct SelectStructurePlugin;

impl Plugin for SelectStructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_hex_menu)
            .add_system(select_hex.pipe(handle_selection));
    }
}

/// A marker component for any element of a hex menu.
#[derive(Component)]
struct HexMenu;

/// An error that can occur when selecting items from a hex menu.
#[derive(PartialEq, Debug)]
enum HexMenuError {
    /// The menu action is not yet released.
    NotYetReleased,
    /// No item was selected.
    NoSelection,
}

/// The location of the items in the hex menu.
#[derive(Resource)]
struct HexMenuArrangement {
    /// A simple mapping from position to contents.
    ///
    /// If entries are missing, the action will be cancelled if released there.
    map: HashMap<Hex, StructureId>,
    /// The size of each hex in pixels
    hex_size: f32,
}

fn spawn_hex_menu(
    mut commands: Commands,
    actions: Res<ActionState<PlayerAction>>,
    hex_menu_arrangement: Res<HexMenuArrangement>,
) {
}

fn select_hex(
    hex_wedges: Query<&Transform, With<HexMenu>>,
    cursor_pos: Res<CursorPos>,
) -> Result<StructureData, HexMenuError> {
    todo!()
}

fn handle_selection(
    In(result): In<Result<StructureData, HexMenuError>>,
    mut clipboard: ResMut<Clipboard>,
    hex_wedges: Query<Entity, With<HexMenu>>,
    mut commands: Commands,
) {
    if result != Err(HexMenuError::NotYetReleased) {
        for entity in hex_wedges.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    match result {
        Ok(data) => {
            clipboard.set(Some(data));
        }
        Err(HexMenuError::NoSelection) => {
            clipboard.set(None);
        }
        _ => (),
    }
}
