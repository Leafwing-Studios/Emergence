//! A reusable gesture-based selector.

use bevy::{prelude::*, utils::HashMap};
use hexx::{Hex, HexLayout, HexOrientation};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    graphics::palette::ui::MENU_NEUTRAL_COLOR,
    player_interaction::{cursor::CursorPos, PlayerAction},
};

use core::fmt::Debug;
use core::hash::Hash;

use super::ui_assets::{Icons, UiElements};

/// A marker component for any element of a hex menu.
#[derive(Component)]
pub(super) struct HexMenu;

/// An error that can occur when selecting items from a hex menu.
#[derive(PartialEq, Debug)]
pub(super) enum HexMenuError {
    /// No item was selected.
    NoSelection {
        /// Is the action complete?
        complete: bool,
    },
    /// No menu exists.
    NoMenu,
}

/// The location of the items in the hex menu.
#[derive(Resource)]
pub(super) struct HexMenuArrangement<D> {
    /// A simple mapping from position to contents.
    ///
    /// If entries are missing, the action will be cancelled if released there.
    content_map: HashMap<Hex, D>,
    /// The collection of menu icon entities at each hex coordinate
    icon_map: HashMap<Hex, Entity>,
    /// The collection of menu background entities at each hex coordinate
    background_map: HashMap<Hex, Entity>,
    /// The geometry of the hex grid
    layout: HexLayout,
}

impl<D: Clone> HexMenuArrangement<D> {
    /// Computes the hex that corresponds to the cursor position.
    fn get_hex(&self, cursor_pos: Vec2) -> Hex {
        self.layout.world_pos_to_hex(cursor_pos)
    }

    /// Fetches the element under the cursor.
    fn get_item(&self, cursor_pos: Vec2) -> Option<D> {
        let hex = self.get_hex(cursor_pos);
        self.content_map.get(&hex).cloned()
    }

    /// The collection of menu background entities at each hex coordinate
    pub(super) fn background_map(&self) -> &HashMap<Hex, Entity> {
        &self.background_map
    }
}

/// The data corresponding to one element of the hex menu.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct HexMenuElement<D> {
    /// The payload of this element.
    data: D,
    /// The hex coordinate of the menu that this corresponds to.
    hex: Hex,
    /// Is the action complete?
    complete: bool,
}

impl<D> HexMenuElement<D> {
    /// The data stored in this element.
    pub(super) fn data(&self) -> &D {
        &self.data
    }

    /// The hex coordinate of the menu that this corresponds to.
    pub(super) fn hex(&self) -> Hex {
        self.hex
    }

    /// Is the action complete?
    pub(super) fn is_complete(&self) -> bool {
        self.complete
    }
}

/// Select a hexagon from the hex menu.
pub(super) fn select_hex<D: Choice>(
    cursor_pos: Res<CursorPos>,
    hex_menu_arrangement: Option<Res<HexMenuArrangement<D>>>,
    actions: Res<ActionState<PlayerAction>>,
) -> Result<HexMenuElement<D>, HexMenuError> {
    if let Some(arrangement) = hex_menu_arrangement {
        let complete = actions.released(D::ACTIVATION);

        if let Some(cursor_pos) = cursor_pos.maybe_screen_pos() {
            if let Some(data) = arrangement.get_item(cursor_pos) {
                Ok(HexMenuElement {
                    data,
                    hex: arrangement.get_hex(cursor_pos),
                    complete,
                })
            } else {
                // Nothing found on lookup
                Err(HexMenuError::NoSelection { complete })
            }
        } else {
            // No cursor
            Err(HexMenuError::NoSelection { complete })
        }
    } else {
        // No menu exists
        Err(HexMenuError::NoMenu)
    }
}

/// A choice that can be used in a wheel menu.
pub(super) trait Choice: Clone + Hash + Eq + Send + Sync + 'static {
    /// The action that is pressed to bring up this wheel menu
    const ACTIVATION: PlayerAction;
}

/// The list of choices available to use in a menu.
#[derive(Resource)]
pub(super) struct AvailableChoices<D> {
    /// The backing ordered [`Vec`]
    pub(super) choices: Vec<D>,
}

impl<D> Default for AvailableChoices<D> {
    fn default() -> Self {
        Self {
            choices: Vec::new(),
        }
    }
}

/// Creates a new hex menu.
pub(super) fn spawn_hex_menu<D: Choice>(
    mut commands: Commands,
    actions: Res<ActionState<PlayerAction>>,
    cursor_pos: Res<CursorPos>,
    ui_elements: Res<UiElements>,
    available_choices: Res<AvailableChoices<D>>,
    icons: Res<Icons<D>>,
) {
    /// The size of the hexes used in this menu.
    const HEX_SIZE: f32 = 64.0;
    if actions.just_pressed(D::ACTIVATION) {
        if let Some(cursor_pos) = cursor_pos.maybe_screen_pos() {
            let mut arrangement = HexMenuArrangement {
                content_map: HashMap::default(),
                icon_map: HashMap::default(),
                background_map: HashMap::default(),
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

            // Center is reserved for easy cancellation.
            let mut hexes =
                Hex::ZERO.custom_spiral_range(1..range, hexx::Direction::BottomRight, true);

            let variants: Vec<_> = Vec::from_iter(available_choices.choices.iter().cloned());

            for data in variants {
                if let Some(hex) = hexes.next() {
                    // Content
                    arrangement.content_map.insert(hex, data.clone());
                    // Icon
                    let icon_entity = commands
                        .spawn(HexMenuIconBundle::new(
                            data,
                            hex,
                            &icons,
                            &arrangement.layout,
                        ))
                        .id();
                    arrangement.icon_map.insert(hex, icon_entity);
                    // Background
                    let background_entity = commands
                        .spawn(HexMenuBackgroundBundle::new(
                            hex,
                            &arrangement.layout,
                            &ui_elements.hex_menu_background,
                        ))
                        .id();
                    arrangement.background_map.insert(hex, background_entity);
                } else {
                    // Just give up rather than panic if too many entries are found
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
    /// Bundle that stores the icon to use
    image_bundle: ImageBundle,
}

impl HexMenuIconBundle {
    /// Create a new icon with the appropriate positioning and appearance.
    fn new<D: Hash + Eq + Send + Sync + 'static>(
        data: D,
        hex: Hex,
        icons: &Icons<D>,
        layout: &HexLayout,
    ) -> Self {
        // Correct for center vs corner positioning
        let half_cell = Vec2 {
            x: layout.hex_size.x / 2.,
            y: layout.hex_size.y / 2.,
        };
        let screen_pos: Vec2 = layout.hex_to_world_pos(hex) - half_cell;

        let image_bundle = ImageBundle {
            image: UiImage {
                texture: icons.get(data),
                flip_x: false,
                flip_y: false,
            },
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
            // Render above the background
            z_index: ZIndex::Global(1),
            ..Default::default()
        };

        HexMenuIconBundle {
            hex_menu: HexMenu,
            image_bundle,
        }
    }
}

/// The background for each menu entry
#[derive(Bundle)]
struct HexMenuBackgroundBundle {
    /// Marker component
    hex_menu: HexMenu,
    /// Background image
    image_bundle: ImageBundle,
}

impl HexMenuBackgroundBundle {
    /// Create a new icon with the appropriate positioning and appearance.
    fn new(hex: Hex, layout: &HexLayout, texture: &Handle<Image>) -> Self {
        /// We must scale these background tiles so they fully tile the background.
        /// Per <https://www.redblobgames.com/grids/hexagons/#basics> (and experimentation)
        /// that means we need a factor of 2
        const BACKGROUND_SCALING_FACTOR: f32 = 2.;
        let width = BACKGROUND_SCALING_FACTOR * layout.hex_size.x;
        let height = BACKGROUND_SCALING_FACTOR * layout.hex_size.y;

        // Correct for center vs corner positioning
        let half_cell = Vec2 {
            x: width / 2.,
            y: height / 2.,
        };
        let screen_pos: Vec2 = layout.hex_to_world_pos(hex) - half_cell;

        let image_bundle = ImageBundle {
            background_color: BackgroundColor(MENU_NEUTRAL_COLOR),
            image: UiImage {
                texture: texture.clone_weak(),
                flip_x: false,
                flip_y: false,
            },
            style: Style {
                position: UiRect {
                    left: Val::Px(screen_pos.x),
                    bottom: Val::Px(screen_pos.y),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                size: Size::new(Val::Px(width), Val::Px(height)),
                ..Default::default()
            },
            // Render below the icon
            z_index: ZIndex::Global(0),
            ..Default::default()
        };

        HexMenuBackgroundBundle {
            hex_menu: HexMenu,
            image_bundle,
        }
    }
}
