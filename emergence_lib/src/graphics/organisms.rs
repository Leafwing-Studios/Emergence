//! The [`OrganismTilemap`] manages visualization of organisms.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

pub use world_query::*;

/// Enumerates organism sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSpriteIndex {
    /// Sprite for an Ant
    Ant,
    /// Sprite for a Plant
    Plant,
    /// Sprite for fungi
    Fungi,
}

impl SpriteIndex for OrganismSpriteIndex {
    const ROOT_PATH: &'static str = "organisms";

    fn leaf_path(&self) -> &'static str {
        match self {
            OrganismSpriteIndex::Ant => "tile-ant.png",
            OrganismSpriteIndex::Fungi => "tile-fungus.png",
            OrganismSpriteIndex::Plant => "tile-plant.png",
        }
    }
}

/// Marker component for entity that manages visualization of organisms.
///
/// The organism tilemap lies on top of the [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap), and
/// keeps track of visualizations of organisms at terrain locations. It is congruent to
/// [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap) in grid size and tile size (for now). Later,
/// we might find it useful to use a different tile size, but the grid size will always remain the
/// same as that of [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap).
#[derive(Component, Clone, Copy, Debug)]
pub struct OrganismsTilemap;

impl TilemapMarker for OrganismsTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 1.0;
    type Index = OrganismSpriteIndex;
}
