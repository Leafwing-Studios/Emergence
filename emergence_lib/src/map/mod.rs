use bevy::ecs::system::Resource;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::TilePos;

/// The number of tiles from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;

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

    /// Computes the total diameter from end-to-end of the game world
    #[inline]
    pub const fn diameter(&self) -> u32 {
        2 * self.radius + 1
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

pub struct MapPlugin;
