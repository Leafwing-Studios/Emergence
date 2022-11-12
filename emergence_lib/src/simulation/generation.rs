use crate::graphics::position::HexNeighbors;
use crate::graphics::MAP_COORD_SYSTEM;
use crate::terrain::terrain_types::TerrainType;
use bevy::app::{App, Plugin, StartupStage};
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::generate_hexagon;
use bevy_ecs_tilemap::tiles::TilePos;
use std::collections::HashMap;

/// Controls world generation strategy
#[derive(Clone)]
pub struct GenerationConfig {
    /// Radius of the map.
    pub map_radius: u32,
    /// Initial number of ants.
    pub n_ant: usize,
    /// Initial number of plants.
    pub n_plant: usize,
    /// Initial number of fungi.
    pub n_fungi: usize,
    /// Relative probability of generating graphics of each terrain type.
    pub terrain_weights: HashMap<TerrainType, f32>,
}

/// Generate the world.
pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationConfig>()
            .init_resource::<PositionCache>()
            // This inserts the `MapGeometry` resource, and so needs to run in an earlier stage
            .add_startup_system_to_stage(StartupStage::PreStartup, configure_map_geometry)
            .add_startup_system_to_stage(StartupStage::Startup, generate_terrain)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_starting_organisms)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_debug_labels);
    }
}

pub struct PositionCache {
    grid_positions: HashMap<TilePos, HexNeighbors<TilePos>>,
}

impl PositionCache {
    pub fn new() {
        let grid_positions = generate_hexagon(
            AxialPos::from_tile_pos_given_coord_system(&map_geometry.center(), MAP_COORD_SYSTEM),
            config.map_radius,
        )
        .into_iter()
        .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(MAP_COORD_SYSTEM));
    }
}
