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

use bevy_console::*;

use bevy_inspector_egui::WorldInspectorPlugin;
use console::{print_to_log, PrintToLog};

pub mod console;
pub mod debug_ui;

/// Creates a global resource that can be used to toggle actively displayed debug tools.
#[derive(Clone, Resource, Component, Copy, Debug, PartialEq, Eq)]
pub struct DebugInfo {
    /// Toggle global access to developer tools
    pub dev_mode: bool,
    /// Toggle developer console
    pub enable_console: bool,
    /// Toggle the debug tile labels
    pub show_tile_label: bool,
    /// Toggle render info
    pub show_fps_info: bool,
    /// Toggle displaying the egui inspector
    pub show_inspector: bool,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            dev_mode: true,
            enable_console: true,
            show_tile_label: true,
            show_fps_info: true,
            show_inspector: true,
        }
    }
}
///
/// Adds an instance of `bevy_console`, basic console commands, `bevy-inspector-egui`,
/// and basic performance information like fps and frame counting.
pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(ConsolePlugin)
            .add_console_command::<PrintToLog, _>(print_to_log);
    }
}
