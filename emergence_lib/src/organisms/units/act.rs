//! Unit behaviour simulation

use crate::{
    organisms::units::pathfinding::get_weighted_position,
    player_interaction::abilities::IntentAbility,
};

use crate::curves::BottomClampedLine;
use crate::signals::emitters::Emitter;
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::index::MapIndex;
use crate::simulation::map::MapPositions;
use crate::simulation::pathfinding::PassabilityCache;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use super::{SignalTransducer, Unit, UnitTimer};

/// Pathfinding for ants.
fn wander(
    position: &TilePos,
    map_positions: &MapPositions,
    passable_filters: &PassabilityCache,
    map_signals: &MapIndex<TileSignals>,
    sensor: &SignalTransducer<BottomClampedLine>,
) -> TilePos {
    let signals_to_weight = |tile_signals: &TileSignals| {
        sensor.signal_to_weight(
            tile_signals.get(&Emitter::Ability(IntentAbility::Lure)),
            tile_signals.get(&Emitter::Ability(IntentAbility::Warning)),
        )
    };

    let position_patch = map_positions.get_patch(position).unwrap();
    let filter_patch = passable_filters.get_patch(position).unwrap();
    let valid_possibilities = position_patch.apply_filter(filter_patch, false).cloned();
    let signals_patch = map_signals.get_patch(position).unwrap();

    let target = get_weighted_position(&valid_possibilities, signals_patch, signals_to_weight);

    target.unwrap_or(*position)
}

/// System modelling ant behaviour.
#[allow(clippy::too_many_arguments)]
pub(super) fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut unit_query: Query<(&Unit, &mut TilePos)>,
    map_positions: Res<MapPositions>,
    passable_filters: Res<PassabilityCache>,
    map_signals: Res<MapIndex<TileSignals>>,
    sensor: Res<SignalTransducer<BottomClampedLine>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in unit_query.iter_mut() {
            *position = wander(
                &position,
                &map_positions,
                &passable_filters,
                &map_signals,
                &sensor,
            );
        }
    }
}
