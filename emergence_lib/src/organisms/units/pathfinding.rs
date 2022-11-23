//! Utilities to support organism pathfinding.
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::neighbors::HexNeighbors;
use crate::simulation::map::resources::MapData;
use bevy_ecs_tilemap::tiles::TilePos;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

/// A tile position with an associated weight. Useful for making weighted selections from a
/// set of tile positions.
#[derive(Clone, Copy, Debug)]
pub struct WeightedTilePos {
    /// Weight associated with the tile position.
    ///
    /// **Important:** This must be non-negative.
    weight: f32,
    /// Tile position that is being assigned a weight.
    position: TilePos,
}

/// Select an adjacent neighboring tile at random, based on the provided weight function.
///
/// Returns [`None`] if and only if no such tile exists.
pub fn get_weighted_neighbor<SignalsToWeight>(
    passable_neighbors: &HexNeighbors<TilePos>,
    neighbor_signals: &HexNeighbors<MapData<TileSignals>>,
    signals_to_weight: SignalsToWeight,
) -> Option<TilePos>
where
    SignalsToWeight: Fn(&TileSignals) -> f32,
{
    let mut rng = thread_rng();

    HexNeighbors::weighted_neighbors(passable_neighbors, neighbor_signals, signals_to_weight)
        .choose_random(&mut rng)
        .map(|weighted_position| weighted_position.position)
}

impl HexNeighbors<WeightedTilePos> {
    /// Returns the set of neighboring cells, weighted according to signal values.
    pub fn weighted_neighbors<SignalsToWeight>(
        passable_neighbors: &HexNeighbors<TilePos>,
        neighbor_signals: &HexNeighbors<MapData<TileSignals>>,
        signals_to_weight: SignalsToWeight,
    ) -> HexNeighbors<WeightedTilePos>
    where
        SignalsToWeight: Fn(&TileSignals) -> f32,
    {
        let f = |direction| {
            let position = *passable_neighbors.get(direction)?;
            let signals = neighbor_signals.get(direction)?;
            let weight = signals_to_weight(&signals.read());
            Some(WeightedTilePos { position, weight })
        };

        HexNeighbors::<WeightedTilePos>::from_directional_closure(f)
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
