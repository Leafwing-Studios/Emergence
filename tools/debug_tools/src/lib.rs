//! Debugging dev tools for Emergence.
//!
//! Keybindings for the developer info toggles:
//! - `dev_mode` is Ctrl+Shift+D.
//! - `show_tile`_labels is Ctrl+Shift+T.
//! - `show_fps_info` is Ctrl+Shift+V.
//! - `show_inspector` is Ctrl+Shift+I.
//! These keybindings were chosen because the average person will not want to touch these very
//! often. Primary, non-modifier keys should be for main gameplay keys.

use bevy::{
    asset::AssetServer,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{
        default, Color, Commands, Component, KeyCode, Plugin, Query, Res, Resource, TextBundle,
        With,
    },
    text::{Text, TextSection, TextStyle},
    time::Time,
    ui::{PositionType, Style, UiRect, Val},
};
use leafwing_input_manager::prelude::*;

// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// use bevy_console::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// use console::{print_to_log, PrintToLog};

// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// pub mod console;
pub mod debug_ui;

// Whichever version of bevy_egui is used by the inspector, make that available to other users of
// debug_tools
pub use bevy_inspector_egui::bevy_egui;

/// Creates a global resource that can be used to toggle actively displayed debug tools.
#[derive(Clone, Resource, Component, Copy, Debug, PartialEq, Eq)]
pub struct DebugInfo {
    /// Toggle global access to developer tools
    pub dev_mode: bool,
    // TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
    // /// Toggle developer console
    // pub enable_console: bool,
    /// Toggle the debug tile labels
    pub show_tile_labels: bool,
    /// Toggle render info
    pub show_fps_info: bool,
    /// Toggle displaying the egui inspector
    pub show_inspector: bool,
}

impl DebugInfo {
    /// Change all the values in this [`DebugInfo`] to be enabled
    pub fn enable(&mut self) {
        self.dev_mode = true;
        self.show_tile_labels = true;
        self.show_fps_info = true;
        self.show_inspector = true;
    }

    /// Change all the values in this [`DebugInfo`] to be disabled
    pub fn disable(&mut self) {
        self.dev_mode = false;
        self.show_tile_labels = false;
        self.show_fps_info = false;
        self.show_inspector = false;
    }
}

// TODO: It might be good later in development or closer to release to make the default disabled
impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            dev_mode: true,
            // TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
            // enable_console: true,
            show_tile_labels: true,
            show_fps_info: true,
            show_inspector: true,
        }
    }
}
/// Adds an instance of `bevy_console`, basic console commands, `bevy-inspector-egui`,
/// and basic performance information like fps and frame counting.
pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugInfo>()
            .add_plugin(InputManagerPlugin::<DevAction>::default())
            .add_systems(Update, debug_ui::show_debug_info);
        // TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
        // .add_plugin(ConsolePlugin)
        // .add_console_command::<PrintToLog, _>(print_to_log);
    }
}

/// Enumerates the actions a developer can take.
#[derive(Actionlike, Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DevAction {
    /// Toggle the overall developer mode setting
    ToggleDevMode,
    // TODO: make debug labels
    /// Toggle tilemap labels tools
    ToggleTileLabels,
    /// Toggle rendering info
    ToggleInfoText,
    /// Toggle the inspector
    ToggleInspector,
}

/// Interface for developer controls
pub struct DevControls {
    /// Toggle the dev mode
    pub toggle_dev_mode: UserInput,
    /// Toggle the tile label
    pub toggle_tile_labels: UserInput,
    /// Toggle the fps ui
    pub toggle_fps: UserInput,
    /// Toggle the inspector
    pub toggle_inspector: UserInput,
}

/// Add default developer controls
impl Default for DevControls {
    fn default() -> Self {
        Self {
            toggle_dev_mode: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::D]),
            toggle_tile_labels: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::T]),
            toggle_fps: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::V]),
            toggle_inspector: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::I]),
        }
    }
}
