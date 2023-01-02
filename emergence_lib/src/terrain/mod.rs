//! Generating and representing terrain as game objects.

pub mod components;
pub mod entity_map;

use crate as emergence_lib;

use crate::simulation::pathfinding::Impassable;
use crate::terrain::components::{HighTerrain, PlainTerrain, RockyTerrain};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;

use bevy_ecs_tilemap::tiles::TilePos;
use emergence_macros::IterableEnum;

/// Available terrain types.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainType {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is rocky, and thus difficult to traverse.
    Rocky,
    /// Terrain that has higher altitude compared to others.
    High,
}

impl TerrainType {
    /// Instantiates an entity bundled with components necessary to characterize terrain
    pub fn instantiate(&self, commands: &mut Commands, position: &TilePos) -> Entity {
        let mut builder = commands.spawn_empty();

        builder.insert(*position);
        match self {
            TerrainType::Plain => {
                builder.insert(PlainTerrain);
            }
            TerrainType::Rocky => {
                builder.insert(RockyTerrain);
                builder.insert(Impassable);
            }
            TerrainType::High => {
                builder.insert(HighTerrain);
            }
        }
        builder.insert(*self);
        builder.id()
    }
}
