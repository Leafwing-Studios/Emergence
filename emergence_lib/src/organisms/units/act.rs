//! Unit behaviour simulation

use crate::organisms::units::pathfinding::get_weighted_neighbors;

use crate::curves::BottomClampedLine;
use crate::graphics::organisms::OrganismStorage;
use crate::graphics::organisms::OrganismStorageItem;
use crate::graphics::terrain::TerrainStorage;
use crate::graphics::terrain::TerrainStorageItem;
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::{MapGeometry, MapPositions};
use crate::simulation::pathfinding::{HexNeighbors, PathfindingImpassable};
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::TilePos;

use super::{PheromoneTransducer, Unit, UnitTimer};

/// Pathfinding for ants.
fn wander(
    position: &TilePos,
    impassable_query: &Query<&PathfindingImpassable>,
    tile_signals_query: &Query<&TileSignals>,
    pheromone_sensor: &PheromoneTransducer<BottomClampedLine>,
    neighbors: HexNeighbors<TilePos>,
) -> TilePos {
    let signals_to_weight = |tile_signals: &TileSignals| {
        pheromone_sensor.signal_to_weight(
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneAttract)),
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneRepulse)),
        )
    };
    let target = get_weighted_neighbors(
        position,
        impassable_query,
        tile_signals_query,
        signals_to_weight,
        neighbors,
    );

    target.unwrap_or(*position)
}

/// System modelling ant behaviour.
#[allow(clippy::too_many_arguments)]
pub(super) fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut unit_query: Query<(&Unit, &mut TilePos)>,
    impassable_query: Query<&TilePos, With<PathfindingImpassable>>,
    tile_signals_query: Query<&TileSignals>,
    pheromone_sensor: Res<PheromoneTransducer<BottomClampedLine>>,
    map_position_cache: Res<MapPositions>,
) {
    let impassable_tiles = HashSet::from_iter(impassable_query.iter().copied());

    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in unit_query.iter_mut() {
            *position = wander(
                &position,
                &impassable_query,
                &tile_signals_query,
                &pheromone_sensor,
                map_position_cache.get_neighbors(&position).unwrap(),
            );
        }
    }
}
