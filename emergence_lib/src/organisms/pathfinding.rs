use crate::terrain::generation::{OrganismTilemap, TerrainTilemap};
use crate::terrain::ImpassableTerrain;
use crate::tiles::position::HexNeighborPositions;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use rand::thread_rng;

/// Select a passable, adjacent neighboring tile at random.
///
/// Returns [`None`] if and only if no such tile exists.
pub fn get_random_passable_neighbor(
    current_pos: &TilePos,
    impassable_query: &Query<&ImpassableTerrain>,
    terrain_tilemap_query: &Query<&TileStorage, With<TerrainTilemap>>,
    organism_tilemap_query: &Query<&TileStorage, With<OrganismTilemap>>,
    map_size: &TilemapSize,
) -> Option<TilePos> {
    let mut rng = thread_rng();
    let terrain_tile_storage = terrain_tilemap_query.single();
    let organism_tile_storage = organism_tilemap_query.single();

    HexNeighborPositions::get_passable_neighbors(
        current_pos,
        terrain_tile_storage,
        organism_tile_storage,
        impassable_query,
        map_size,
    )
    .choose_random(&mut rng)
}
