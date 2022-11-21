//! Manages the game world's grid

use crate::simulation::generation::{GenerationConfig, MAP_RADIUS};
use crate::simulation::map_data::HexNeighbors;
use crate::simulation::space::HexNeighbors;
use bevy::ecs::system::Resource;
use bevy::prelude::{Commands, Res};
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

    /// Computes the number of positions that exist in this map
    pub const fn n_positions(&self) -> usize {
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

    /// Gets the radius of the map
    #[inline]
    pub const fn radius(&self) -> u32 {
        self.radius
    }

    /// Checks to see if the given tile position lies within the map
    #[inline]
    pub const fn check_inclusion(&self, tile_pos: &TilePos) -> bool {
        let delta_x = (tile_pos.x as isize - self.center.x as isize).abs() as u32;
        if delta_x < self.radius + 1 {
            let delta_y = (tile_pos.y as isize - self.center.y as isize).abs() as u32;
            if delta_y < self.radius + 1 {
                true
            } else {
                false
            }
        } else {
            false
        }
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

/// Resource caching tile positions, neighbors for a fixed map size
#[derive(Resource, Default)]
pub struct MapPositions {
    /// Number of positions that are expected
    n_positions: usize,
    /// Positions that exist in the map
    positions: Vec<TilePos>,
    /// Map caching hex neighbors with each tile position
    neighbors: HashMap<TilePos, HexNeighbors<TilePos>>,
}

impl MapPositions {
    /// Creates with capacity `n_positions`
    pub fn new(&self, n_positions: usize) -> MapPositions {
        MapPositions {
            n_positions,
            positions: Vec::with_capacity(n_positions),
            neighbors: HashMap::with_capacity(n_positions),
        }
    }

    /// Get all cached tile positions
    pub fn iter_positions(&self) -> impl Iterator<Item = &TilePos> + '_ {
        self.positions.iter()
    }

    /// Get neighbors associated with a given tile position, if it exists in the cache
    pub fn get_neighbors(&self, tile_pos: &TilePos) -> Option<&HexNeighbors<TilePos>> {
        self.neighbors.get(tile_pos)
    }

    /// Get the number of positions that are managed by this structure
    pub fn n_positions(&self) -> usize {
        self.n_positions
    }
}

/// Populate the [`MapPositionCache`] resource with positions and neighbors.
pub fn populate_position_cache(mut commands: Commands, map_geometry: Res<MapGeometry>) {
    let mut map_cache = MapPositions::new(map_geometry.n_positions());

    let center = map_geometry.center();
    let radius = map_geometry.radius();
    // When using HexCoordSystem::Row, TilePos is the same as AxialPos, so we can get away with
    // unchecked/fast conversions between AxialPos and TilePos
    for tile_pos in generate_hexagon(AxialPos::from(&center), radius)
        .into_iter()
        .map(|axial_pos| axial_pos.as_tile_pos_unchecked())
    {
        map_cache.positions.push(tile_pos);
        map_cache.neighbors.insert(
            tile_pos,
            HexNeighbors::get_neighbors(&tile_pos, &map_geometry),
        );
    }

    commands.insert_resource(map_cache);
}
