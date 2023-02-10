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
    /// Computes the hex that corresponds to the cursor position.
    fn get_hex(&self, cursor_pos: Vec2) -> Hex {
        self.layout.world_pos_to_hex(cursor_pos)
    }

    /// Fetches the element under the cursor.
    fn get_item(&self, cursor_pos: Vec2) -> Option<StructureId> {
        let hex = self.get_hex(cursor_pos);
        self.content_map.get(&hex).cloned()
    }
}

/// Creates a new hex menu.
fn spawn_hex_menu(
    mut commands: Commands,
    actions: Res<ActionState<PlayerAction>>,
    cursor_pos: Res<CursorPos>,
    structure_info: Res<StructureInfo>,
) {
    /// The size of the hexes used in this menu.
    const HEX_SIZE: f32 = 64.0;

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

            let hexes = Hex::ZERO.custom_spiral_range(range, hexx::Direction::BottomRight, true);

            for (i, structure_id) in structure_info.keys().enumerate() {
                // Center is reserved for easy cancellation.
                // Just give up rather than panic if too many entities are found
                if let Some(&hex) = hexes.get(i + 1) {
                    arrangement.content_map.insert(hex, structure_id.clone());
                    let icon_entity = commands
                        .spawn(HexMenuIconBundle::new(
                            structure_id,
                            hex,
                            &structure_info,
                            &arrangement.layout,
                        ))
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
    /// Create a new icon with the appropriate positioning and appearance.
    fn new(
        structure_id: &StructureId,
        hex: Hex,
        structure_info: &StructureInfo,
        layout: &HexLayout,
    ) -> Self {
        let color = structure_info.color(structure_id);
        // Correct for center vs corner positioning
        let half_cell = Vec2 {
            x: layout.hex_size.x / 2.,
            y: layout.hex_size.y / 2.,
        };
        let screen_pos: Vec2 = layout.hex_to_world_pos(hex) - half_cell;

        let image_bundle = ImageBundle {
            background_color: BackgroundColor(color),
            style: Style {
                position: UiRect {
                    left: Val::Px(screen_pos.x),
                    bottom: Val::Px(screen_pos.y),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                size: Size::new(Val::Px(layout.hex_size.x), Val::Px(layout.hex_size.y)),
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

/// Select a hexagon from the hex menu.
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

/// Set the selected structure based on the results of the hex menu.
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
