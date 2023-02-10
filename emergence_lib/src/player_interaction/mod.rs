//! Tools for the player to interact with the world

use bevy::prelude::*;
use leafwing_input_manager::{
    prelude::{ActionState, DualAxis, InputManagerPlugin, InputMap, VirtualDPad},
    user_input::{Modifier, UserInput},
    Actionlike,
};

pub(crate) mod abilities;
pub(crate) mod camera;
pub(crate) mod clipboard;
pub(crate) mod cursor;
pub(crate) mod intent;
pub(crate) mod organism_details;
pub(crate) mod tile_selection;
pub(crate) mod zoning;

/// All of the code needed for users to interact with the simulation.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_plugin(camera::CameraPlugin)
            .add_plugin(abilities::AbilitiesPlugin)
            .add_plugin(cursor::CursorPlugin)
            .add_plugin(intent::IntentPlugin)
            .add_plugin(organism_details::DetailsPlugin)
            .add_plugin(tile_selection::TileSelectionPlugin)
            .add_plugin(clipboard::ClipboardPlugin)
            .add_plugin(zoning::ZoningPlugin);

        #[cfg(feature = "debug_tools")]
        app.add_plugin(debug_tools::DebugToolsPlugin);
    }
}

/// Public system sets for player interaction, used for system ordering and config
#[derive(SystemLabel, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum InteractionSystem {
    /// Moves the camera
    MoveCamera,
    /// Cursor position is set
    ComputeCursorPos,
    /// Tiles are selected
    SelectTiles,
    /// Held structure(s) are selected
    SetClipboard,
    /// Replenishes the [`IntentPool`](intent::IntentPool) of the hive mind
    ReplenishIntent,
    /// Apply zoning to tiles
    ApplyZoning,
    /// Use intent-spending abilities
    UseAbilities,
    /// Spawn and despawn ghosts
    ManageGhosts,
    /// Updates information about the hovered entities
    HoverDetails,
}

/// Actions that the player can take to modify the game world or their view of it.
///
/// This should only store actions that need a dedicated keybinding.
#[derive(Actionlike, Clone, Debug)]
pub(crate) enum PlayerAction {
    /// Selects a tile or group of tiles.
    Select,
    /// Deselects a tile or group of tiles.
    Deselect,
    /// Modifies the selection / deselection to be sequential.
    Multiple,
    /// Modifies the selection to cover a hexagonal area.
    Area,
    /// Modifies the selection to cover a line between the start and end of the selection.
    Line,
    /// Selects the structure on the tile under the player's cursor.
    ///
    /// If there is no structure there, the player's selection is cleared.
    Pipette,
    /// Sets the zoning of all currently selected tiles to the currently selected structure.
    ///
    /// If no structure is selected, any zoning will be removed.
    Zone,
    /// Sets the zoning of all currently selected tiles to [`Zoning::None`](zoning::Zoning::None).
    ///
    /// If no structure is selected, any zoning will be removed.
    ClearZoning,
    /// Removes all structures from the clipboard.
    ClearClipboard,
    /// Rotates the conents of the clipboard counterclockwise.
    RotateClipboardLeft,
    /// Rotates the contents of the clipboard clockwise.
    RotateClipboardRight,
    /// Move the camera from side to side
    Pan,
    /// Reveal less of the map by moving the camera closer
    ZoomIn,
    /// Reveal more of the map by pulling the camera away
    ZoomOut,
    /// Rotates the camera counterclockwise
    RotateCameraLeft,
    /// Rotates the camera clockwise
    RotateCameraRight,
}

impl PlayerAction {
    /// The default keybindings for mouse and keyboard.
    fn kbm_binding(&self) -> UserInput {
        match self {
            PlayerAction::Select => MouseButton::Left.into(),
            PlayerAction::Deselect => MouseButton::Right.into(),
            PlayerAction::Multiple => Modifier::Shift.into(),
            PlayerAction::Area => Modifier::Control.into(),
            PlayerAction::Line => Modifier::Alt.into(),
            PlayerAction::Pipette => KeyCode::Q.into(),
            PlayerAction::Zone => KeyCode::Space.into(),
            PlayerAction::ClearZoning => KeyCode::Back.into(),
            PlayerAction::ClearClipboard => KeyCode::Escape.into(),
            PlayerAction::RotateClipboardLeft => UserInput::modified(Modifier::Shift, KeyCode::R),
            PlayerAction::RotateClipboardRight => KeyCode::R.into(),
            PlayerAction::Pan => VirtualDPad::wasd().into(),
            PlayerAction::ZoomIn => KeyCode::Plus.into(),
            PlayerAction::ZoomOut => KeyCode::Minus.into(),
            PlayerAction::RotateCameraLeft => KeyCode::Z.into(),
            PlayerAction::RotateCameraRight => KeyCode::C.into(),
        }
    }

    /// The default keybindings for gamepads.
    fn gamepad_binding(&self) -> UserInput {
        use GamepadButtonType::*;
        let camera_modifier = RightTrigger2;

        match self {
            PlayerAction::Select => South.into(),
            PlayerAction::Deselect => East.into(),
            PlayerAction::Multiple => RightTrigger.into(),
            PlayerAction::Area => LeftTrigger.into(),
            PlayerAction::Line => LeftTrigger2.into(),
            PlayerAction::Pipette => West.into(),
            PlayerAction::Zone => North.into(),
            PlayerAction::ClearZoning => DPadUp.into(),
            PlayerAction::ClearClipboard => DPadDown.into(),
            PlayerAction::RotateClipboardLeft => DPadLeft.into(),
            PlayerAction::RotateClipboardRight => DPadRight.into(),
            PlayerAction::Pan => DualAxis::right_stick().into(),
            PlayerAction::ZoomIn => UserInput::chord([camera_modifier, DPadUp]),
            PlayerAction::ZoomOut => UserInput::chord([camera_modifier, DPadDown]),
            PlayerAction::RotateCameraLeft => UserInput::chord([camera_modifier, DPadLeft.into()]),
            PlayerAction::RotateCameraRight => {
                UserInput::chord([camera_modifier, DPadRight.into()])
            }
        }
    }

    /// The default key bindings
    fn default_input_map() -> InputMap<PlayerAction> {
        let mut input_map = InputMap::default();

        for variant in PlayerAction::variants() {
            input_map.insert(variant.kbm_binding(), variant.clone());
            input_map.insert(variant.gamepad_binding(), variant);
        }
        input_map
    }
}
