//! Tools for the player to interact with the world

use bevy::prelude::*;
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
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
        app.add_plugin(InputManagerPlugin::<SelectionAction>::default())
            .init_resource::<ActionState<SelectionAction>>()
            .insert_resource(SelectionAction::default_input_map())
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

/// Actions that can be used to select tiles.
///
/// If a tile is not selected, it will be added to the selection.
/// If it is already selected, it will be removed from the selection.
#[derive(Actionlike, Clone, Debug)]
pub(crate) enum SelectionAction {
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
    /// Rotates the contents of the clipboard clockwise.
    RotateClipboardRight,
    /// Rotates the conents of the clipboard counterclockwise.
    RotateClipboardLeft,
}

impl SelectionAction {
    /// The default keybindings for mouse and keyboard.
    fn kbm_binding(&self) -> UserInput {
        match self {
            SelectionAction::Select => MouseButton::Left.into(),
            SelectionAction::Deselect => MouseButton::Right.into(),
            SelectionAction::Multiple => Modifier::Shift.into(),
            SelectionAction::Area => Modifier::Control.into(),
            SelectionAction::Line => Modifier::Alt.into(),
            SelectionAction::Pipette => KeyCode::Q.into(),
            SelectionAction::Zone => KeyCode::Space.into(),
            SelectionAction::ClearZoning => KeyCode::Back.into(),
            SelectionAction::ClearClipboard => KeyCode::Escape.into(),
            SelectionAction::RotateClipboardRight => KeyCode::R.into(),
            SelectionAction::RotateClipboardLeft => {
                UserInput::modified(Modifier::Shift, KeyCode::R)
            }
        }
    }

    /// The default key bindings
    fn default_input_map() -> InputMap<SelectionAction> {
        let mut input_map = InputMap::default();

        for variant in SelectionAction::variants() {
            input_map.insert(variant.kbm_binding(), variant);
        }
        input_map
    }
}
