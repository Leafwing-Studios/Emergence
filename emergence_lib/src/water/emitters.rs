//! Emitters produce water from nothing.

use bevy::prelude::*;

use crate::{
    simulation::{
        geometry::{Height, MapGeometry, TilePos},
        time::InGameTime,
    },
    structures::Landmark,
};

use super::{WaterConfig, WaterTable};

// FIXME: not all landmarks should produce water
/// Creates water from each emitter.
pub(super) fn produce_water_from_emitters(
    water_config: Res<WaterConfig>,
    query: Query<(&WaterEmitter, &TilePos)>,
    mut water_table: ResMut<WaterTable>,
    fixed_time: Res<FixedTime>,
    in_game_time: Res<InGameTime>,
    map_geometry: Res<MapGeometry>,
) {
    let elapsed_time = fixed_time.period.as_secs_f32() / in_game_time.seconds_per_day();

    for (water_emitter, &tile_pos) in query.iter() {
        let emitter_pressure = water_emitter.pressure;
        let surface_water_height = map_geometry
            .get_surface_water_height(tile_pos)
            .unwrap_or_default();

        // The rate of flow should gradually decrease as the water level rises.
        // Eventually, the rate of flow reaches zero when the water level is equal to the emitter's pressure.
        let remaining_pressure = (emitter_pressure - surface_water_height).max(Height::ZERO);

        // Use a seperate scaling factor for the water production rate,
        // so then we can tweak the water production rate without affecting the max depth.
        let produced_water = water_config.emission_rate * remaining_pressure.0 * elapsed_time;
        water_table.add(tile_pos, produced_water);
    }
}

/// An entity that produces water.
#[derive(Component, Debug)]
pub(super) struct WaterEmitter {
    /// The maximum height of water that this emitter can be covered with before it stops producing.
    ///
    /// This controls the rate of water production: higher values produce more water.
    pressure: Height,
}

pub(super) fn add_water_emitters(
    mut commands: Commands,
    water_config: Res<WaterConfig>,
    // TODO: not all landmarks should produce water
    query: Query<Entity, (With<Landmark>, Without<WaterEmitter>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(WaterEmitter {
            pressure: water_config.emission_pressure,
        });
    }
}
