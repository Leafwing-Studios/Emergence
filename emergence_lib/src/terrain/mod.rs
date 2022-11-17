//! Generating and representing terrain as game objects.
pub mod marker;

use crate as emergence_lib;
use crate::enum_iter::IterableEnum;
use crate::terrain::marker::{HighTerrain, ImpassableTerrain, PlainTerrain};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use emergence_macros::IterableEnum;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;
use crate::simulation::pathfinding::PathfindingImpassable;

/// Available terrain types.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainType {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is impassable.
    Impassable,
    /// Terrain that has higher altitude compared to others.
    High,
}

impl TerrainType {
    /// Instantiates an entity bundled with components necessary to characterize terrain
    pub fn instantiate(&self, commands: &mut Commands, position: TilePos) -> Entity {
        let mut builder = commands.spawn_empty();

        builder.insert(position);
        match self {
            TerrainType::Plain => builder.insert(PlainTerrain),
            TerrainType::Impassable => builder.insert(ImpassableTerrain {
                impassable: PathfindingImpassable
            }),
            TerrainType::High => builder.insert(HighTerrain),
        };
        builder.insert(*self);
        builder.id()
    }

    /// Choose a random terrain tile based on the given weights
    pub fn choose_random<R: Rng + ?Sized>(
        rng: &mut R,
        weights: &HashMap<TerrainType, f32>,
    ) -> Result<TerrainType, WeightedError> {
        TerrainType::variants()
            .collect::<Vec<TerrainType>>()
            .choose_weighted(rng, |terrain_type| {
                weights.get(terrain_type).copied().unwrap_or_default()
            })
            .copied()
    }
}
