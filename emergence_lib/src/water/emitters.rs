//! Emitters produce water from nothing.

use bevy::prelude::*;

use crate::{
    simulation::{
        geometry::{Height, TilePos},
        time::InGameTime,
    },
    structures::Landmark,
};

use super::WaterTable;

// FIXME: not all landmarks should produce water
pub(super) fn produce_water_from_emitters(
    query: Query<&TilePos, With<Landmark>>,
    mut water_table: ResMut<WaterTable>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
) {
    /// The amount of water that is deposited per day on the tile of each water emitter.
    const WATER_PER_DAY: Height = Height(10.0);

    let water_per_second = WATER_PER_DAY.0 / in_game_time.seconds_per_day();
    let elapsed_time = fixed_time.period.as_secs_f32();
    let water_rate = Height(water_per_second * elapsed_time);

    for tile_pos in query.iter() {
        water_table.add(*tile_pos, water_rate);
    }
}
