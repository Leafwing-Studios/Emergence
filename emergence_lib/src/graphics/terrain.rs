//! The [`TerrainTilemap`] manages visualization of terrain.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapLike;
use bevy::prelude::Component;
use bevy_ecs_tilemap::prelude::{TilemapGridSize, TilemapTileSize};
use emergence_macros::IterableEnum;
use std::path::PathBuf;

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
    const ROOT_FOLDER: &'static str = "terrain";

    fn leaf_path(&self) -> PathBuf {
        match self {
            TerrainSprite::High => "tile-high.png".into(),
            TerrainSprite::Rocky => "tile-rocky.png".into(),
            TerrainSprite::Plain => "tile-plain.png".into(),
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

impl TilemapLike for TerrainTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const GRID_SIZE: Option<TilemapGridSize> = None;
    const MAP_Z: f32 = 0.0;
    type Index = TerrainSprite;
}
