//! Emitters produce water from nothing.

use bevy::prelude::*;

use crate::{
    simulation::{
        geometry::{Height, MapGeometry, TilePos, Volume},
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
        let surface_water_height = map_geometry
            .get_surface_water_height(tile_pos)
            .unwrap_or_default();

        // Use a seperate scaling factor for the water production rate,
        // so then we can tweak the water production rate without affecting the max depth.
        let produced_water = water_emitter
            .current_water_production(surface_water_height, &water_config)
            * elapsed_time;
        water_table.add(tile_pos, produced_water);
    }
}

/// An entity that produces water.
#[derive(Component, Debug, Clone)]
pub(crate) struct WaterEmitter {
    /// The maximum height of water that this emitter can be covered with before it stops producing.
    ///
    /// This controls the rate of water production: higher values produce more water.
    pressure: Height,
}

impl WaterEmitter {
    /// The maximum height of water that this emitter can be covered with before it stops producing.
    pub(crate) fn pressure(&self) -> Height {
        self.pressure
    }

    /// Computes the current amount of water that this emitter can produce, in tiles per day.
    pub(crate) fn current_water_production(
        &self,
        surface_water_height: Height,
        water_config: &WaterConfig,
    ) -> Volume {
        // If the water level is below the surface, it should be treated as 0,
        // as it does not apply any pressure to the emitter due to its weight.
        assert!(surface_water_height >= Height::ZERO);

        // The rate of flow should gradually decrease as the water level rises.
        // Eventually, the rate of flow reaches zero when the water level is equal to the emitter's pressure.
        let remaining_pressure =
            Volume::from_height((self.pressure - surface_water_height).max(Height::ZERO));
        remaining_pressure.0 * water_config.emission_rate
    }

    /// Computes the maximum amount of water that this emitter can produce in a single day.
    pub(crate) fn max_water_production(&self, water_config: &WaterConfig) -> Volume {
        water_config.emission_rate
    }
}

/// Adds the [`WaterEmitter`] component to the structures that need it.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A water emitter that produces water at a constant rate.
    const TEST_EMITTER: WaterEmitter = WaterEmitter {
        pressure: Height(1.0),
    };

    /// A simple test configuration.
    const TEST_CONFIG: WaterConfig = WaterConfig {
        emission_pressure: Height(1.0),
        emission_rate: Volume(1.0),
        ..WaterConfig::NULL
    };

    #[test]
    fn water_emitter_does_not_emit_when_covered() {
        let water_production =
            TEST_EMITTER.current_water_production(TEST_EMITTER.pressure(), &TEST_CONFIG);
        assert_eq!(water_production, Volume::ZERO);

        let water_production = TEST_EMITTER
            .current_water_production(TEST_EMITTER.pressure() + Height(0.5), &TEST_CONFIG);
        assert_eq!(water_production, Volume::ZERO);
    }

    #[test]
    fn water_emitter_emits_when_uncovered() {
        let water_production = TEST_EMITTER.current_water_production(Height::ZERO, &TEST_CONFIG);
        assert!(water_production > Volume::ZERO);

        let max_water_production = TEST_EMITTER.max_water_production(&TEST_CONFIG);
        assert!(water_production == max_water_production);

        let water_production = TEST_EMITTER.current_water_production(Height(0.5), &TEST_CONFIG);
        assert!(water_production > Volume::ZERO);
    }

    #[test]
    fn water_emitter_emits_more_slowly_as_it_becomes_covered() {
        let water_production_1 = TEST_EMITTER.current_water_production(Height(0.1), &TEST_CONFIG);
        let water_production_covered_2 =
            TEST_EMITTER.current_water_production(Height(0.2), &TEST_CONFIG);
        let water_production_covered_3 =
            TEST_EMITTER.current_water_production(Height(0.3), &TEST_CONFIG);

        assert!(water_production_1 > water_production_covered_2);
        assert!(water_production_covered_2 > water_production_covered_3);
    }
}
