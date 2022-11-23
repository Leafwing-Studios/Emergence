//! Unit behaviour simulation

use crate::organisms::units::pathfinding::get_weighted_neighbor;

use crate::curves::BottomClampedLine;
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::neighbors::HexNeighbors;
use crate::simulation::map::resources::MapResource;
use crate::simulation::map::MapPositions;
use crate::simulation::pathfinding::PassableFilters;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use super::{PheromoneTransducer, Unit, UnitTimer};

/// Pathfinding for ants.
fn wander(
    position: &TilePos,
    map_positions: &MapPositions,
    passable_filters: &PassableFilters,
    map_signals: &MapResource<TileSignals>,
    pheromone_sensor: &PheromoneTransducer<BottomClampedLine>,
) -> TilePos {
    let signals_to_weight = |tile_signals: &TileSignals| {
        pheromone_sensor.signal_to_weight(
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneAttract)),
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneRepulse)),
        )
    };

    let neighbors = map_positions.get_neighbors(position).unwrap();
    let passable_filter = passable_filters.get_neighbors(position).unwrap();
    let passable_neighbors: HexNeighbors<TilePos> =
        neighbors.apply_filter(passable_filter, false).cloned();
    let neighbor_signals = map_signals.get_neighbors(position).unwrap();

    let target = get_weighted_neighbor(&passable_neighbors, neighbor_signals, signals_to_weight);

    target.unwrap_or(*position)
}

/// System modelling ant behaviour.
#[allow(clippy::too_many_arguments)]
pub(super) fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut unit_query: Query<(&Unit, &mut TilePos)>,
    map_positions: Res<MapPositions>,
    passable_filters: Res<PassableFilters>,
    map_signals: Res<MapResource<TileSignals>>,
    pheromone_sensor: Res<PheromoneTransducer<BottomClampedLine>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in unit_query.iter_mut() {
            *position = wander(
                &position,
                &map_positions,
                &passable_filters,
                &map_signals,
                &pheromone_sensor,
            );
        }
    }
}
