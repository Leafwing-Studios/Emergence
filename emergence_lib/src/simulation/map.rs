//! Manages the game world's grid

use crate::graphics::MAP_COORD_SYSTEM;
use crate::simulation::generation::GenerationConfig;
use crate::simulation::pathfinding::HexNeighbors;
use bevy::ecs::system::Resource;
use bevy::prelude::{Commands, Res, ResMut};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::prelude::axial::AxialPos;
use bevy_ecs_tilemap::prelude::generate_hexagon;
use bevy_ecs_tilemap::tiles::TilePos;

/// Resource that stores information regarding the size of the game map.
#[derive(Resource, Debug)]
pub struct MapGeometry {
    /// The radius, in graphics, of the map
    radius: u32,
    /// The location of the central tile
    center: TilePos,
    /// The [`TilemapSize`] of the map
    size: TilemapSize,
}

impl Default for MapGeometry {
    fn default() -> Self {
        MapGeometry::new(MAP_RADIUS)
    }
}

impl MapGeometry {
    /// Constructs a new [`MapGeometry`] for a `radius`.
    pub const fn new(radius: u32) -> Self {
        MapGeometry {
            radius,
            center: TilePos {
                x: radius + 1,
                y: radius + 1,
            },
            size: TilemapSize {
                x: 2 * radius + 1,
                y: 2 * radius + 1,
            },
        }
    }

    /// Computes the number of tiles that exist in this map
    pub const fn n_tiles(&self) -> usize {
        1 + 6 * ((self.radius * (self.radius + 1)) / 2) as usize
    }

    /// Computes the total diameter from end-to-end of the game world
    #[inline]
    pub const fn diameter(&self) -> u32 {
        self.size.x
    }

    /// Computes the [`TilemapSize`] of the game world
    #[inline]
    pub const fn size(&self) -> TilemapSize {
        self.size
    }

    /// Computes the [`TilePos`] of the tile at the center of this map.
    ///
    /// This is not (0,0) as `bevy_ecs_tilemap` works with `u32` coordinates.
    #[inline]
    pub const fn center(&self) -> TilePos {
        self.center
    }
}

impl From<&GenerationConfig> for MapGeometry {
    fn from(config: &GenerationConfig) -> MapGeometry {
        MapGeometry::new(config.map_radius)
    }
}

/// Initialize the [`MapGeometry`] resource according to [`GenerationConfig`].
pub fn configure_map_geometry(mut commands: Commands, config: Res<GenerationConfig>) {
    let map_geometry: MapGeometry = (&*config).into();

    commands.insert_resource(map_geometry);
}

#[derive(Resource, Default)]
/// Resource caching available map postions
pub struct MapPositionsCache {
    /// Map caching hex neighbors with each tile position
    map: HashMap<TilePos, HexNeighbors<TilePos>>,
}

impl MapPositionsCache {
    /// Get all cached tile positions
    pub fn positions(&self) -> impl Iterator<Item = TilePos> + '_ {
        self.map.keys().copied()
    }

    /// Get neighbors associated with a given tile position, if it exists in the cache
    pub fn get_neighbors(&self, tile_pos: TilePos) -> Option<&HexNeighbors<TilePos>> {
        self.map.get(&tile_pos)
    }
}

/// Populate the [`MapPositionCache`] resource with positions and neighbors.
pub fn populate_position_cache(mut commands: Commands, map_geometry: Res<MapGeometry>) {
    let mut map_positions = MapPositionsCache {
        map: HashMap::with_capacity(map_geometry.n_tiles()),
    };

    for tile_pos in generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&map_geometry.center(), MAP_COORD_SYSTEM),
        map_geometry.radius,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(MAP_COORD_SYSTEM))
    {
        map_positions.map.insert(
            tile_pos,
            HexNeighbors::get_neighbors(&tile_pos, &map_geometry.size),
        );
    }

    commands.insert_resource(map_positions);
}
