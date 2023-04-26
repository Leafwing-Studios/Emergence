//! Logic about water movement and behavior.
//!
//! Code for how it should be rendered belongs in the `graphics` module,
//! while code for how it can be used typically belongs in `structures`.

use bevy::prelude::*;

use crate::simulation::{
    geometry::{Height, MapGeometry, TilePos},
    time::InGameTime,
    SimulationSet,
};

/// A plugin that handles water movement and behavior.
pub(super) struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTable>().add_systems(
            (tides, update_surface_water_map_geometry)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// The height of the water.
///
/// This can be underground, at ground level, or above ground.
/// If it is above ground, it will pool on top of the tile it is on.
#[derive(Resource)]
pub struct WaterTable {
    /// The height of the water table.
    base_height: Height,
    /// The single global height of the water table.
    height: Height,
}

impl Default for WaterTable {
    fn default() -> Self {
        /// The height of the water table at the start of the game.
        const BASE_HEIGHT: Height = Height(2);

        Self {
            base_height: BASE_HEIGHT,
            height: BASE_HEIGHT,
        }
    }
}

/// Varies the height of the water table over time.
// TODO: this should only vary the ocean height, not the whole water table
fn tides(in_game_time: Res<InGameTime>, mut water_table: ResMut<WaterTable>) {
    /// How much the water table height varies over time.
    ///
    /// This is in units of [`Height`], and the total variation is twice this.
    const TIDAL_SCALE: f32 = 2.0;

    let tidal_offset = in_game_time.elapsed_days().sin() * TIDAL_SCALE;
    water_table.height = water_table.base_height + Height(tidal_offset.round() as u8);
}

/// Computes how much water is on the surface of each tile.
fn update_surface_water_map_geometry(
    mut map_geometry: ResMut<MapGeometry>,
    water_table: Res<WaterTable>,
) {
    // Collect out to avoid borrow checker pain
    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        let tile_height = map_geometry.get_height(tile_pos).unwrap();

        if water_table.height > tile_height {
            map_geometry.add_surface_water(tile_pos, water_table.height - tile_height);
        } else {
            map_geometry.remove_surface_water(tile_pos);
        }
    }
}
