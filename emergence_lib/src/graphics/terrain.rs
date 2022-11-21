//! The [`TerrainTilemap`] manages visualization of terrain.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::prelude::Component;
use bevy_ecs_tilemap::prelude::TilemapTileSize;
use emergence_macros::IterableEnum;

pub use world_query::*;

/// Enumerates terrain sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainSpriteIndex {
    /// Sprite for high terrain,
    High,
    /// Sprite for impassable terrain
    Impassable,
    /// Sprite for plain terrain
    Plain,
}

impl SpriteIndex for TerrainSpriteIndex {
    const ROOT_PATH: &'static str = "terrain";

    fn leaf_path(&self) -> &'static str {
        match self {
            TerrainSpriteIndex::High => "tile-high.png",
            TerrainSpriteIndex::Impassable => "tile-impassable.png",
            TerrainSpriteIndex::Plain => "tile-plain.png",
        }
    }
}

/// Marker component for entity that manages visualization of terrain.
///
/// See also, the [`OrganismTilemap`](crate::graphics::organisms::OrganismTilemap), which lies on top of the
/// terrain tilemap, and manages visualization of organisms.
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainTilemap;

impl TilemapMarker for TerrainTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 0.0;
    type Index = TerrainSpriteIndex;
}
