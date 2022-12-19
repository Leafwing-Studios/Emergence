//! Debugging dev tools for Emergence.

use bevy::{
    asset::AssetServer,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{
        default, Color, Commands, Component, Plugin, Query, ReflectComponent, Res, Resource,
        TextBundle, With,
    }, // Input, KeyCode, // add back to allow for toggling the fps display
    // Transform, // add back for tile labels
    reflect::Reflect,
    text::{Text, TextSection, TextStyle}, // Text2dBundle, TextAlignment, //add back for tile labels
    time::Time,
    ui::{PositionType, Style, UiRect, Val},
    // DefaultPlugins,
};

use bevy_console::*; // add when console is implemented

use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use console::{print_to_log, PrintToLog};

pub mod console;
pub mod debug_ui;

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
// Generate Debug Tools plugin
pub struct DebugToolsPlugin;

// tells bevy-inspector-egui how to display the struct in the world inspector
#[derive(Inspectable, Component)]
struct InspectableType;

// registers the type in the `bevy_reflect` machinery, so that even without implementing `Inspectable` we can display the struct fields
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct ReflectedType;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .register_inspectable::<InspectableType>()
            .register_type::<ReflectedType>()
            .add_plugin(ConsolePlugin)
            .add_console_command::<PrintToLog, _>(print_to_log);
    }
}
