use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapGridSize, TilemapId, TilemapSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileTexture};

pub mod position;

/// The number of tiles from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;
/// The number of tiles from edge to opposite edge of the map
pub const MAP_DIAMETER: u32 = 2 * MAP_RADIUS + 1;
/// The [`TilemapSize`] of the complete world map
pub const MAP_SIZE: TilemapSize = TilemapSize {
    x: MAP_DIAMETER,
    y: MAP_DIAMETER,
};

/// The [`TilePos`] that defines the center of this map
pub const MAP_CENTER: TilePos = TilePos {
    x: MAP_RADIUS + 1,
    y: MAP_RADIUS + 1,
};

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

/// We use a hexagonal map with "point-topped" (row oriented) tiles, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// A type that can be transformed into a tile that is compatible with [`bevy_ecs_tilemap`].
pub trait IntoTileBundle {
    /// The corresponding [`TileTexture`].
    fn tile_texture(&self) -> TileTexture;

    /// The asset path to the [`TileTexture`].
    fn tile_texture_path(&self) -> &'static str;

    /// Uses the data stored in `self` to create a new, matching [`TileBundle`].
    fn as_tile_bundle(&self, tilemap_id: TilemapId, position: TilePos) -> TileBundle {
        TileBundle {
            position,
            tilemap_id,
            texture: self.tile_texture(),
            ..Default::default()
        }
    }
}
