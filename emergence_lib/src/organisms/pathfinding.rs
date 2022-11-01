//! Utilities to support organism pathfinding.
use crate::organisms::OrganismStorageItem;
use crate::signals::tile_signals::TileSignals;
use crate::terrain::{ImpassableTerrain, TerrainStorageItem};
use crate::tiles::position::HexNeighbors;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use rand::distributions::WeightedError;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

/// Select a passable, adjacent neighboring tile at random.
///
/// Returns [`None`] if and only if no such tile exists.
pub fn get_random_passable_neighbor(
    current_pos: &TilePos,
    terrain_tile_storage: &TerrainStorageItem,
    organism_tile_storage: &OrganismStorageItem,
    impassable_query: &Query<&ImpassableTerrain>,
    map_size: &TilemapSize,
) -> Option<TilePos> {
    let mut rng = thread_rng();

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
        terrain_tile_storage: &TerrainStorageItem,
        organism_tile_storage: &OrganismStorageItem,
        impassable_query: &Query<&ImpassableTerrain>,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let passable_filter_closure = |pos| {
            // there should be a terrain entity, otherwise the position is not accessible
            let terrain_entity = terrain_tile_storage.storage.get(&pos)?;
            // if the terrain entity we found is impassable, then return None
            let _ = impassable_query.get(terrain_entity).err()?;

            if let Some(organism_entity) = organism_tile_storage.storage.get(&pos) {
                // if organism entity at given tile position is impassable, then return None
                let _ = impassable_query.get(organism_entity).err()?;
            }

            Some(pos)
        };

        HexNeighbors::get_neighbors(tile_pos, map_size).and_then(passable_filter_closure)
    }
}

/// A tile position with an associated weight. Useful for making weighted selections from a
/// set of tile positions.
#[derive(Clone, Copy, Debug)]
pub struct WeightedTilePos {
    /// Weight associated with the tile position.
    ///
    /// **Important:** This must be non-negative.
    weight: f32,
    /// Tile position that is being assigned a weight.
    pos: TilePos,
}

/// Select a passable, adjacent neighboring tile at random.
///
/// Returns [`None`] if and only if no such tile exists.
pub fn get_weighted_random_passable_neighbor<SignalsToWeightClosure>(
    current_pos: &TilePos,
    terrain_tile_storage: &TerrainStorageItem,
    organism_tile_storage: &OrganismStorageItem,
    impassable_query: &Query<&ImpassableTerrain>,
    tile_signals_query: &Query<&TileSignals>,
    signals_to_weight: SignalsToWeightClosure,
    map_size: &TilemapSize,
) -> Option<TilePos>
where
    SignalsToWeightClosure: Fn(&TileSignals) -> f32,
{
    let mut rng = thread_rng();

    HexNeighbors::weighted_passable_neighbors(
        current_pos,
        terrain_tile_storage,
        organism_tile_storage,
        impassable_query,
        tile_signals_query,
        signals_to_weight,
        map_size,
    )
    .choose_random(&mut rng)
    .map(|weighted_pos| weighted_pos.pos)
}

impl HexNeighbors<WeightedTilePos> {
    /// Returns the set of neighboring cells that are passable, weighted according to signal values.
    pub fn weighted_passable_neighbors<SignalsToWeightFn>(
        tile_pos: &TilePos,
        terrain_tile_storage: &TerrainStorageItem,
        organism_tile_storage: &OrganismStorageItem,
        impassable_query: &Query<&ImpassableTerrain>,
        tile_signals_query: &Query<&TileSignals>,
        signals_to_weight: SignalsToWeightFn,
        map_size: &TilemapSize,
    ) -> HexNeighbors<WeightedTilePos>
    where
        SignalsToWeightFn: Fn(&TileSignals) -> f32,
    {
        let passable_neighbors = HexNeighbors::passable_neighbors(
            tile_pos,
            terrain_tile_storage,
            organism_tile_storage,
            impassable_query,
            map_size,
        );

        info!("pos: {tile_pos:?}, passable_neighbors: {passable_neighbors:?}");

        let f = |pos| {
            let tile_entity = terrain_tile_storage.storage.get(pos).unwrap();
            let weight = if let Ok(tile_signals) = tile_signals_query.get(tile_entity) {
                signals_to_weight(tile_signals)
            } else {
                info!("No signal found...");
                1.0
            };

            WeightedTilePos { weight, pos: *pos }
        };
        let weighted_neighbors = passable_neighbors.map_ref(f);
        info!("pos: {tile_pos:?}, weighted_neighbors: {weighted_neighbors:?}");
        weighted_neighbors
    }

    /// Get the entities associated with neighbouring tile positions.
    pub fn entities(&self, tile_storage: &TileStorage) -> HexNeighbors<Entity> {
        let f = |weighted_tile_pos: &WeightedTilePos| tile_storage.get(&weighted_tile_pos.pos);
        self.and_then_ref(f)
    }

    /// Choose a random neighbor
    pub fn choose_random<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<WeightedTilePos> {
        let possible_choices = self.iter().copied().collect::<Vec<WeightedTilePos>>();

        match possible_choices
            .choose_weighted(rng, |weighted_pos| weighted_pos.weight)
            .cloned()
        {
            Ok(weighted_tile_pos) => Some(weighted_tile_pos),
            Err(e) => match e {
                WeightedError::NoItem => None,
                WeightedError::InvalidWeight => {
                    panic!("Encountered invalid weight in choices: {possible_choices:?}")
                }
                WeightedError::AllWeightsZero => None,
                WeightedError::TooMany => {
                    panic!("Too many weights were provided! Choices: {possible_choices:?}")
                }
            },
        }
    }
}
