//! Quickly select which structure to place.

use bevy::{prelude::*, utils::HashMap};
use hexx::{Hex, HexLayout, HexOrientation};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    player_interaction::{
        clipboard::{Clipboard, StructureData},
        cursor::CursorPos,
        PlayerAction,
    },
    simulation::geometry::Facing,
    structures::{StructureId, StructureInfo},
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
    /// The geometry of the hex grid
    layout: HexLayout,
}

impl HexMenuArrangement {
    /// Evaluates the hex that is stored under the
    fn get_hex(&self, cursor_pos: Vec2) -> Hex {
        self.layout.world_pos_to_hex(cursor_pos)
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
    cursor_pos: Res<CursorPos>,
    structure_info: Res<StructureInfo>,
) {
    /// The size of the hexes used in this menu.
    const HEX_SIZE: f32 = 10.0;

    if actions.just_pressed(PlayerAction::SelectStructure) {
        if let Some(cursor_pos) = cursor_pos.maybe_screen_pos() {
            let mut arrangement = HexMenuArrangement {
                content_map: HashMap::default(),
                icon_map: HashMap::default(),
                layout: HexLayout {
                    orientation: HexOrientation::pointy(),
                    origin: cursor_pos,
                    hex_size: Vec2 {
                        x: HEX_SIZE,
                        y: HEX_SIZE,
                    },
                },
            };

            // Any larger than this is quite unwieldy
            let range = 3;

            let hexes = Hex::ZERO.custom_spiral_range(range, hexx::Direction::Top, true);

            for (i, structure_id) in structure_info.keys().enumerate() {
                // Center is reserved for easy cancellation.
                // Just give up rather than panic if too many entities are found
                if let Some(&hex) = hexes.get(i + 1) {
                    arrangement.content_map.insert(hex, structure_id.clone());
                    let screen_pos: Vec2 = arrangement.layout.hex_to_world_pos(hex);
                    let icon_entity = commands
                        .spawn(HexMenuIconBundle::new(structure_id, screen_pos))
                        .id();
                    arrangement.icon_map.insert(hex, icon_entity);
                } else {
                    warn!("Too many entries in hex menu!");
                }
            }

            commands.insert_resource(arrangement);
        }
    }
}

/// The icon stored presented in a hex menu
#[derive(Bundle)]
struct HexMenuIconBundle {
    /// Marker component
    hex_menu: HexMenu,
    /// Small image of structure
    image_bundle: ImageBundle,
}

impl HexMenuIconBundle {
    fn new(structure_id: &StructureId, screen_pos: Vec2) -> Self {
        // TODO: customize these
        let image_bundle = ImageBundle {
            background_color: BackgroundColor(Color::RED),
            style: Style {
                position: UiRect {
                    right: Val::Px(screen_pos.x),
                    top: Val::Px(screen_pos.y),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        HexMenuIconBundle {
            hex_menu: HexMenu,
            image_bundle,
        }
    }
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
