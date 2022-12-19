//! Debugging dev tools for Emergence.
//!

use bevy::{
    asset::AssetServer,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{
        default, info, Color, Commands, Component, Plugin, Query, ReflectComponent, Res, Resource,
        TextBundle, With,
    },
    prelude::{default, info, Color, Commands, Component, Query, Res, TextBundle, Transform, With},
    reflect::Reflect,
    text::{Text, Text2dBundle, TextAlignment, TextSection, TextStyle},
    text::{Text, TextSection, TextStyle},
    time::Time,
    ui::{PositionType, Style, UiRect, Val},
};

use bevy_console::*;

use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use console::{log_command, LogCommand};

pub mod console;
pub mod debug_ui;

#[derive(Clone, Resource, Copy, Debug, PartialEq, Eq)]
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
            .register_inspectable::<InspectableType>()
            .register_type::<ReflectedType>()
            .add_plugin(ConsolePlugin)
            .add_console_command::<LogCommand, _>(log_command);
    }
}
