use crate::position::HexNeighborPositions;
use crate::terrain::ImpassableTerrain;
use crate::tilemap::{OrganismTilemap, TerrainTilemap};
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use rand::thread_rng;

pub fn get_random_passable_neighbor(
    current_pos: &TilePos,
    impassable_query: &Query<&ImpassableTerrain>,
    terrain_tilemap_query: &Query<&TileStorage, With<TerrainTilemap>>,
    organism_tilemap_query: &Query<&TileStorage, With<OrganismTilemap>>,
) -> Option<TilePos> {
    let mut rng = thread_rng();
    for terrain_tile_storage in terrain_tilemap_query.iter() {
        for organism_tile_storage in organism_tilemap_query.iter() {
            return HexNeighborPositions::get_passable_neighbors(
                current_pos,
                terrain_tile_storage,
                organism_tile_storage,
                impassable_query,
            )
            .choose_random(&mut rng);
        }
    }
    None
}
