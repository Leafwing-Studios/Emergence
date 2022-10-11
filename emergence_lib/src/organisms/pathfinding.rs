use crate::terrain::generation::{OrganismTilemap, TerrainTilemap};
use crate::terrain::ImpassableTerrain;
use crate::tiles::position::HexNeighbors;
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

    HexNeighbors::passable_neighbors(
        current_pos,
        terrain_tile_storage,
        organism_tile_storage,
        impassable_query,
        map_size,
    )
    .choose_random(&mut rng)
}

impl HexNeighbors<TilePos> {
    /// Returns the set of neighboring cells that units can walk through
    pub fn passable_neighbors(
        tile_pos: &TilePos,
        terrain_tile_storage: &TileStorage,
        organism_tile_storage: &TileStorage,
        impassable_query: &Query<&ImpassableTerrain>,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let passable_filter_closure = |pos| {
            // there should be a terrain entity, otherwise the position is not accessible
            let terrain_entity = terrain_tile_storage.get(&pos)?;
            // if the terrain entity we found is impassable, then return None
            let _ = impassable_query.get(terrain_entity).err()?;

            if let Some(organism_entity) = organism_tile_storage.get(&pos) {
                // if organism entity at given tile position is impassable, then return None
                let _ = impassable_query.get(organism_entity).err()?;
            }

            Some(pos)
        };

        HexNeighbors::get_neighbors(tile_pos, map_size).and_then(passable_filter_closure)
    }
}
