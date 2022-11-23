//! The [`OrganismsTilemap`] manages visualization of organisms.

use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates organism sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSprite {
    /// Sprite for an Ant
    Ant,
    /// Sprite for a Plant
    Plant,
    /// Sprite for fungi
    Fungi,
}

impl SpriteIndex for OrganismSprite {
    const ROOT_PATH: &'static str = "organisms";

    fn leaf_path(&self) -> &'static str {
        match self {
            OrganismSprite::Ant => "tile-ant.png",
            OrganismSprite::Fungi => "tile-fungus.png",
            OrganismSprite::Plant => "tile-plant.png",
        }
    }
}

/// Marker component for entity that manages visualization of organisms.
///
/// See also:
/// * [`ProduceTilemap`](crate::graphics::produce::ProduceTilemap), which lies on top of the
/// [`OrganismsTilemap`], and manages visualization of organisms
/// * [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap), which lies on below the
/// [`OrganismsTilemap`], and manages visualization of terrain entities
#[derive(Component, Clone, Copy, Debug)]
pub struct OrganismsTilemap;

impl TilemapMarker for OrganismsTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 1.0;
    type Index = OrganismSprite;
}
