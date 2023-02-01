//! Generating and representing terrain as game objects.

use bevy::prelude::*;

use crate as emergence_lib;

use crate::simulation::geometry::TilePos;
use bevy::ecs::component::Component;

use emergence_macros::IterableEnum;

/// Available terrain types.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum Terrain {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is rocky, and thus difficult to traverse.
    Rocky,
    /// Terrain that has higher altitude compared to others.
    High,
}

/// All of the components needed to define a piece of terrain.
#[derive(Bundle)]
pub struct TerrainBundle {
    terrain_type: Terrain,
    tile_pos: TilePos,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    pub fn new(terrain_type: Terrain, tile_pos: TilePos) -> Self {
        TerrainBundle {
            terrain_type,
            tile_pos,
        }
    }
}
