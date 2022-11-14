use crate::organisms::units::pathfinding::get_weighted_random_passable_neighbor;

use crate::curves::BottomClampedLine;
use crate::graphics::organisms::OrganismStorage;
use crate::graphics::organisms::OrganismStorageItem;
use crate::graphics::terrain::TerrainStorage;
use crate::graphics::terrain::TerrainStorageItem;
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::tile_signals::TileSignals;
use crate::terrain::{ImpassableTerrain, MapGeometry};
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::TilePos;

use super::{PheromoneTransducer, Unit, UnitTimer};

/// Pathfinding for ants.
fn wander(
    position: &TilePos,
    terrain_tile_storage: &TerrainStorageItem,
    organism_tile_storage: &OrganismStorageItem,
    impassable_query: &Query<&ImpassableTerrain>,
    tile_signals_query: &Query<&TileSignals>,
    pheromone_sensor: &PheromoneTransducer<BottomClampedLine>,
    map_size: &TilemapSize,
) -> TilePos {
    let signals_to_weight = |tile_signals: &TileSignals| {
        pheromone_sensor.signal_to_weight(
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneAttract)),
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneRepulse)),
        )
    };
    let target = get_weighted_random_passable_neighbor(
        position,
        terrain_tile_storage,
        organism_tile_storage,
        impassable_query,
        tile_signals_query,
        signals_to_weight,
        map_size,
    );

    target.unwrap_or(*position)
}

/// System modelling ant behaviour.
#[allow(clippy::too_many_arguments)]
pub(super) fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut query: Query<(&Unit, &mut TilePos)>,
    impassable_query: Query<&ImpassableTerrain>,
    terrain_storage_query: Query<TerrainStorage>,
    organism_storage_query: Query<OrganismStorage>,
    tile_signals_query: Query<&TileSignals>,
    pheromone_sensor: Res<PheromoneTransducer<BottomClampedLine>>,
    map_geometry: Res<MapGeometry>,
) {
    let terrain_tile_storage = terrain_storage_query.single();
    let organism_tile_storage = organism_storage_query.single();
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in query.iter_mut() {
            *position = wander(
                &position,
                &terrain_tile_storage,
                &organism_tile_storage,
                &impassable_query,
                &tile_signals_query,
                &pheromone_sensor,
                &map_geometry.size(),
            );
        }
    }
}
