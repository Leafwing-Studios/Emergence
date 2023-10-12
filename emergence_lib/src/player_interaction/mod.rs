//! Tools for the player to interact with the world

use crate::enum_iter::IterableEnum;
use crate::{self as emergence_lib};
use bevy::prelude::*;
use emergence_macros::IterableEnum;

use leafwing_input_manager::{
    prelude::{ActionState, DualAxis, InputManagerPlugin, InputMap, VirtualDPad},
    user_input::{Modifier, UserInput},
    Actionlike,
};

use crate::world_gen::WorldGenState;

pub(crate) mod camera;
pub(crate) mod clipboard;
pub(crate) mod picking;
pub(crate) mod selection;

/// All of the code needed for users to interact with the simulation.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_plugins(camera::CameraPlugin)
            .add_plugins(picking::PickingPlugin)
            .add_plugins(selection::SelectionPlugin)
            .add_plugins(clipboard::ClipboardPlugin)
            .configure_set(
                Update,
                PlayerModifiesWorld.run_if(in_state(WorldGenState::Complete)),
            );

        for variant in InteractionSystem::variants() {
            app.configure_set(Update, variant.run_if(in_state(WorldGenState::Complete)));
        }
    }
}

/// Public system sets for player interaction, used for system ordering and config
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug, IterableEnum)]
pub(crate) enum InteractionSystem {
    /// Moves the camera
    MoveCamera,
    /// Cursor position is set
    ComputeCursorPos,
    /// Tiles are selected
    SelectTiles,
    /// Held structure(s) are selected
    SetClipboard,
    /// Apply zoning to tiles
    ApplyZoning,
    /// Spawn and despawn ghosts
    ManagePreviews,
}

/// A system set for all actions that the player can take which modify the game world.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PlayerModifiesWorld;

/// Actions that the player can take to modify the game world or their view of it.
///
/// This should only store actions that need a dedicated keybinding.
#[derive(Actionlike, Reflect, Clone, Debug)]
pub(crate) enum PlayerAction {
    /// Pause or unpause the game.
    TogglePause,
    /// When the clipboard is full, places the clipboard contents on the map.
    ///
    /// When the clipboard is empty, selects a tile or group of tiles.
    UseTool,
    /// When the clipboard is full, clears the clipboard.
    ///
    /// When the clipboard is empty, deselects a tile or group of tiles.
    Deselect,
    /// Increases the radius of the selection by one tile.
    IncreaseSelectionRadius,
    /// Decreases the radius of the selection by one tile.
    DecreaseSelectionRadius,
    /// Modifies the selection / deselection to be sequential.
    Multiple,
    /// Modifies the selection to cover a hexagonal area.
    Area,
    /// Modifies the selection to cover a line between the start and end of the selection.
    Line,
    /// Selects a structure from a wheel menu.
    SelectStructure,
    /// Select a terraforming tool from a wheel menu.
    SelectTerraform,
    /// Select an ability from a wheel menu.
    SelectAbility,
    /// Selects the structure on the tile under the player's cursor.
    ///
    /// If there is no structure there, the player's selection is cleared.
    Copy,
    /// Sets the zoning of all currently selected tiles to the currently selected structure.
    Paste,
    /// Cancels any planned actions (ghosts) selected.
    ClearZoning,
    /// Rotates the contents of the clipboard counterclockwise.
    RotateClipboardLeft,
    /// Rotates the contents of the clipboard clockwise.
    RotateClipboardRight,
    /// Snaps the camera to the selected object
    CenterCameraOnSelection,
    /// Drag the camera with the cursor
    DragCamera,
    /// Move the camera from side to side
    Pan,
    /// Move the cursor around the screen
    MoveCursor,
    /// Reveal less of the map by moving the camera closer
    ZoomIn,
    /// Reveal more of the map by pulling the camera away
    ZoomOut,
    /// Tilts the camera up, towards vertical
    TiltCameraUp,
    /// Tilts the camera down, towards horizontal
    TiltCameraDown,
    /// Rotates the camera counterclockwise
    RotateCameraLeft,
    /// Rotates the camera clockwise
    RotateCameraRight,
    /// Toggles the status overlay
    ToggleStatusInfo,
    /// Toggle the signal strength overlay
    ToggleSignalOverlay,
    /// Show / hide the strongest signal overlay
    ToggleStrongestSignalOverlay,
    /// Show / hide the depth to the water table overlay
    ToggleWaterTableOverlay,
    /// Show / hide the light overlay
    ToggleLightOverlay,
}

impl PlayerAction {
    /// The default keybindings for mouse and keyboard.
    fn kbm_binding(&self) -> UserInput {
        use PlayerAction::*;
        match self {
            TogglePause => KeyCode::Space.into(),
            UseTool => MouseButton::Left.into(),
            Deselect => MouseButton::Right.into(),
            // Plus and Equals are swapped. See: https://github.com/rust-windowing/winit/issues/2682
            IncreaseSelectionRadius => UserInput::modified(Modifier::Control, KeyCode::Equals),
            DecreaseSelectionRadius => UserInput::modified(Modifier::Control, KeyCode::Minus),
            Multiple => Modifier::Shift.into(),
            Area => Modifier::Control.into(),
            Line => Modifier::Alt.into(),
            SelectStructure => KeyCode::Key1.into(),
            SelectTerraform => KeyCode::Key2.into(),
            SelectAbility => KeyCode::Key3.into(),
            Copy => UserInput::modified(Modifier::Control, KeyCode::C),
            Paste => UserInput::modified(Modifier::Control, KeyCode::V),
            ClearZoning => KeyCode::Back.into(),
            RotateClipboardLeft => UserInput::modified(Modifier::Shift, KeyCode::R),
            RotateClipboardRight => KeyCode::R.into(),
            CenterCameraOnSelection => KeyCode::L.into(),
            DragCamera => MouseButton::Middle.into(),
            Pan => VirtualDPad::wasd().into(),
            MoveCursor => VirtualDPad::arrow_keys().into(),
            // Plus and Equals are swapped. See: https://github.com/rust-windowing/winit/issues/2682
            ZoomIn => KeyCode::Equals.into(),
            ZoomOut => KeyCode::Minus.into(),
            // Plus and Equals are swapped. See: https://github.com/rust-windowing/winit/issues/2682
            TiltCameraUp => UserInput::modified(Modifier::Alt, KeyCode::Equals),
            TiltCameraDown => UserInput::modified(Modifier::Alt, KeyCode::Minus),
            RotateCameraLeft => KeyCode::Q.into(),
            RotateCameraRight => KeyCode::E.into(),
            ToggleStatusInfo => KeyCode::F1.into(),
            ToggleSignalOverlay => KeyCode::F2.into(),
            ToggleStrongestSignalOverlay => KeyCode::F3.into(),
            ToggleWaterTableOverlay => KeyCode::F4.into(),
            ToggleLightOverlay => KeyCode::F5.into(),
        }
    }

    /// The default keybindings for gamepads.
    fn gamepad_binding(&self) -> UserInput {
        use GamepadButtonType::*;
        use PlayerAction::*;

        let camera_modifier = RightTrigger2;
        let radius_modifier = LeftTrigger;
        let infovis_modifier = LeftTrigger2;
        let selection_modifier = RightTrigger;

        match self {
            TogglePause => GamepadButtonType::Select.into(),
            PlayerAction::UseTool => South.into(),
            Deselect => East.into(),
            Multiple => RightTrigger.into(),
            IncreaseSelectionRadius => UserInput::chord([radius_modifier, DPadUp]),
            DecreaseSelectionRadius => UserInput::chord([radius_modifier, DPadDown]),
            Area => LeftTrigger.into(),
            Line => LeftTrigger2.into(),
            Copy => West.into(),
            Paste => North.into(),
            ClearZoning => DPadUp.into(),
            SelectStructure => UserInput::chord([selection_modifier, West]),
            SelectTerraform => UserInput::chord([selection_modifier, North]),
            SelectAbility => UserInput::chord([selection_modifier, East]),
            RotateClipboardLeft => DPadLeft.into(),
            RotateClipboardRight => DPadRight.into(),
            CenterCameraOnSelection => GamepadButtonType::LeftThumb.into(),
            DragCamera => GamepadButtonType::RightThumb.into(),
            Pan => DualAxis::left_stick().into(),
            MoveCursor => DualAxis::right_stick().into(),
            ZoomIn => UserInput::chord([camera_modifier, DPadUp]),
            ZoomOut => UserInput::chord([camera_modifier, DPadDown]),
            TiltCameraUp => UserInput::chord([RightTrigger, DPadDown]),
            TiltCameraDown => UserInput::chord([RightTrigger, DPadDown]),
            RotateCameraLeft => UserInput::chord([camera_modifier, DPadLeft]),
            RotateCameraRight => UserInput::chord([camera_modifier, DPadRight]),
            ToggleStatusInfo => UserInput::chord([infovis_modifier, DPadLeft]),
            // FIXME: this should just be removed in favor of forcing cursor control
            ToggleSignalOverlay => UserInput::chord([infovis_modifier, DPadUp]),
            ToggleStrongestSignalOverlay => UserInput::chord([infovis_modifier, DPadRight]),
            ToggleWaterTableOverlay => UserInput::chord([infovis_modifier, DPadDown]),
            ToggleLightOverlay => UserInput::chord([infovis_modifier, DPadUp]),
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
