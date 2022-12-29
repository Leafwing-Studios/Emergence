//! Debugging dev tools for Emergence.

use bevy::{
    asset::AssetServer,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{
        default, Color, Commands, Component, Plugin, Query, Res, Resource, TextBundle, With,
    },
    text::{Text, TextSection, TextStyle},
    time::Time,
    ui::{PositionType, Style, UiRect, Val},
};

// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// use bevy_console::*;

use bevy_inspector_egui::WorldInspectorPlugin;
// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// use console::{print_to_log, PrintToLog};

// TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
// pub mod console;
pub mod debug_ui;

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
            .add_plugin(FrameTimeDiagnosticsPlugin);
        // TODO: see: https://github.com/Leafwing-Studios/Emergence/issues/140
        // .add_plugin(ConsolePlugin)
        // .add_console_command::<PrintToLog, _>(print_to_log);
    }
}
