//! Debugging dev tools for Emergence.
//!

use bevy::{
    asset::AssetServer,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{default, Color, Commands, Component, Query, Res, TextBundle, Transform, With}, // Input, KeyCode, // add back to allow for toggling the fps display
    text::{Text, Text2dBundle, TextAlignment, TextSection, TextStyle},
    time::Time,
    ui::{PositionType, Style, UiRect, Val},
};

use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
// use emergence_lib::graphics::terrain::TerrainTilemap;

pub mod debug_ui;
