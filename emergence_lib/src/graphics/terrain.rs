//! The [`TerrainTilemap`] manages visualization of terrain.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::prelude::Component;
use bevy_ecs_tilemap::prelude::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates terrain sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainSprite {
    /// Sprite for high terrain,
    High,
    /// Sprite for impassable terrain
    Rocky,
    /// Sprite for plain terrain
    Plain,
}

impl SpriteIndex for TerrainSprite {
    const ROOT_PATH: &'static str = "terrain";

    fn leaf_path(&self) -> &'static str {
        match self {
            TerrainSprite::High => "tile-high.png",
            TerrainSprite::Rocky => "tile-rocky.png",
            TerrainSprite::Plain => "tile-plain.png",
        }
    }
}

/// Marker component for entity that manages visualization of terrain.
///
/// See also:
/// * [`ProduceTilemap`](crate::graphics::produce::ProduceTilemap), which lies on top of the
/// [`TerrainTilemap`], and manages visualization of organisms
/// * [`OrganismsTilemap`](crate::graphics::organisms::OrganismsTilemap), which lies on below the
/// [`TerrainTilemap`], and manages visualization of terrain entities
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainTilemap;

impl TilemapMarker for TerrainTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 0.0;
    type Index = TerrainSprite;
}
