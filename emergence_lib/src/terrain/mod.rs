//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

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
    /// Terrain that is unusually muddy.
    Muddy,
}

impl Terrain {
    /// The rendering material associated with this terrain type.
    pub fn material(&self) -> StandardMaterial {
        let base_color = match self {
            Terrain::Plain => Color::BEIGE,
            Terrain::Rocky => Color::GRAY,
            Terrain::Muddy => Color::BISQUE,
        };

        StandardMaterial {
            base_color,
            ..Default::default()
        }
    }
}

/// All of the components needed to define a piece of terrain.
#[derive(Bundle)]
pub struct TerrainBundle {
    /// The type of terrain
    terrain_type: Terrain,
    /// The location of this terrain hex
    tile_pos: TilePos,
    /// Makes the tiles pickable
    raycast_mesh: RaycastMesh<Terrain>,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    pub fn new(terrain_type: Terrain, tile_pos: TilePos) -> Self {
        TerrainBundle {
            terrain_type,
            tile_pos,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
        }
    }
}
