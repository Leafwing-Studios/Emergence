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
    simulation::geometry::Facing,
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
    /// No menu exists.
    NoMenu,
}

/// The location of the items in the hex menu.
#[derive(Resource)]
struct HexMenuArrangement {
    /// A simple mapping from position to contents.
    ///
    /// If entries are missing, the action will be cancelled if released there.
    content_map: HashMap<Hex, StructureId>,
    /// The collection of menu icon entities at each hex coordinate
    icon_map: HashMap<Hex, Entity>,
    /// The size of each hex in pixels
    hex_size: f32,
    /// The origin of this menu, in screen coordinates
    center: Vec2,
}

impl HexMenuArrangement {
    /// Evaluates the hex that is stored under the
    fn get_hex(&self, cursor_pos: Vec2) -> Hex {
        todo!()
    }

    fn get_item(&self, cursor_pos: Vec2) -> Option<StructureId> {
        let hex = self.get_hex(cursor_pos);
        self.content_map.get(&hex).cloned()
    }

    fn get_icon_entity(&self, cursor_pos: Vec2) -> Option<Entity> {
        let hex = self.get_hex(cursor_pos);
        self.icon_map.get(&hex).cloned()
    }
}

fn spawn_hex_menu(
    mut commands: Commands,
    actions: Res<ActionState<PlayerAction>>,
    hex_menu_arrangement: Res<HexMenuArrangement>,
) {
}

fn select_hex(
    cursor_pos: Res<CursorPos>,
    hex_menu_arrangement: Option<Res<HexMenuArrangement>>,
    actions: Res<ActionState<PlayerAction>>,
) -> Result<StructureId, HexMenuError> {
    if let Some(arrangement) = hex_menu_arrangement {
        if actions.released(PlayerAction::SelectStructure) {
            if let Some(cursor_pos) = cursor_pos.maybe_screen_pos() {
                let selection = arrangement.get_item(cursor_pos);
                match selection {
                    Some(item) => Ok(item),
                    None => Err(HexMenuError::NoSelection),
                }
            } else {
                Err(HexMenuError::NoSelection)
            }
        } else {
            Err(HexMenuError::NotYetReleased)
        }
    } else {
        Err(HexMenuError::NoMenu)
    }
}

fn handle_selection(
    In(result): In<Result<StructureId, HexMenuError>>,
    mut clipboard: ResMut<Clipboard>,
    hex_wedges: Query<Entity, With<HexMenu>>,
    mut commands: Commands,
) {
    if result == Err(HexMenuError::NoMenu) || result == Err(HexMenuError::NotYetReleased) {
        return;
    }

    match result {
        Ok(id) => {
            let structure_data = StructureData {
                id,
                facing: Facing::default(),
            };

            clipboard.set(Some(structure_data));
        }
        Err(HexMenuError::NoSelection) => {
            clipboard.set(None);
        }
        _ => (),
    }

    for entity in hex_wedges.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.remove_resource::<HexMenuArrangement>();
}
