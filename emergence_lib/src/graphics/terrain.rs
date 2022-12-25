//! The [`TerrainTilemap`] manages visualization of terrain.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;

use bevy::prelude::Component;

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

/// Marker component for the terrain tilemap
#[derive(Component, Clone, Copy, Debug)]
pub struct TerrainTilemap;
