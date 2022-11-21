//! Utilities to support organism pathfinding.
use crate::signals::tile_signals::TileSignals;
use crate::simulation::pathfinding::HexNeighbors;
use bevy::utils::HashSet;
use bevy_ecs_tilemap::tiles::TilePos;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};


impl HexNeighbors<TilePos> {
    /// Returns the set of neighboring cells that units can walk through
    pub fn passable_neighbors(&self, impassable_tiles: &HashSet<TilePos>) -> HexNeighbors<TilePos> {
        let passable_filter_closure = |pos| (!impassable_tiles.contains(&pos)).then_some(pos);
        self.and_then(passable_filter_closure)
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

/// Select an adjacent neighboring tile at random, based on the provided weight function.
///
/// Returns [`None`] if and only if no such tile exists.
pub fn get_weighted_neighbors<W>(
    neighbor_signals: &HexNeighbors<&TileSignals>,
    signal_to_weight: W,
) -> Option<TilePos>
where
    W: Fn(&TileSignals) -> f32,
{
    let mut rng = thread_rng();

    HexNeighbors::weighted_passable_neighbors(
        current_pos,
        neighbors,
        impassable_tiles,
        tile_signals_query,
        signals_to_weight,
    )
    .choose_random(&mut rng)
    .map(|weighted_pos| weighted_pos.pos)
}

impl HexNeighbors<WeightedTilePos> {
    /// Returns the set of neighboring cells, weighted according to signal values.
    pub fn weighted_neighbors<Transducer>(
        &self,
        neighbor_signals: &HexNeighbors<&TileSignals>,
        signal_transducer: Transducer,
    ) -> HexNeighbors<WeightedTilePos>
    where
        Transducer: Fn(&TileSignals) -> f32,
    {
        self.and_then(|_| )
        let f = |pos| {
            let tile_entity = terrain_tile_storage.storage.get(pos).unwrap();
            let weight = if let Ok(tile_signals) = neighbor_signals.get(tile_entity) {
                signal_transducer(tile_signals)
            } else {
                1.0
            };

            WeightedTilePos { weight, pos: *pos }
        };

        passable_neighbors.map_ref(f)
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
