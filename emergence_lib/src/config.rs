use crate::organisms::OrganismType;
use crate::terrain::TerrainType;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapGridSize, TilemapSize, TilemapTileSize};
use bevy_ecs_tilemap::tiles::TilePos;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

/// The default width of the game window.
pub const WINDOW_WIDTH: f32 = 1920.0;
/// The default height of the game window.
pub const WINDOW_HEIGHT: f32 = 1080.0;

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

/// An [`IndexMap`] of organism images.
pub static ORGANISM_TILE_IMAP: Lazy<IndexMap<OrganismType, &'static str>> = Lazy::new(|| {
    use OrganismType::*;
    indexmap! {
        Ant => "tile-ant.png",
        Fungus => "tile-fungus.png",
        Plant => "tile-plant.png",
    }
});

/// The tile size (hex tile width by hex tile height) in pixels of organism image assets.
pub const ORGANISM_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };

/// The z-coordinate at which organisms are drawn.
///
/// We want the organism tilemap to be layered on top of the terrain tile map.
pub const ORGANISM_TILEMAP_Z: f32 = 1.0;

pub static TERRAIN_TILE_IMAP: Lazy<IndexMap<TerrainType, &'static str>> = Lazy::new(|| {
    use TerrainType::*;
    indexmap! {
        High => "tile-high.png",
        Impassable => "tile-impassable.png",
        Plain => "tile-plain.png",
    }
});

/// The tile size (hex tile width by hex tile height) in pixels of tile image assets.
pub const TERRAIN_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
/// The z-coordinate at which tiles are drawn.
pub const TERRAIN_TILEMAP_Z: f32 = 0.0;

/// The gap in pixels around each tile
pub const TILE_BUFFER: f32 = 0.0;

/// The number of tiles from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;
/// The number of tiles from edge to opposite edge of the map
pub const MAP_DIAMETER: u32 = 2 * MAP_RADIUS + 1;
/// The [`TilemapSize`] of the complete world map
pub const MAP_SIZE: TilemapSize = TilemapSize {
    x: MAP_DIAMETER,
    y: MAP_DIAMETER,
};
/// The coordinate system used in this game
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// The [`TilePos`] that defines the center of this map
pub const MAP_CENTER: TilePos = TilePos {
    x: MAP_RADIUS + 1,
    y: MAP_RADIUS + 1,
};

/// The number of ants initially spawned
pub const N_ANT: usize = 5;
/// The number of plants initially spawned
pub const N_PLANT: usize = 10;
/// The number of fungi initially spawned
pub const N_FUNGI: usize = 10;

/// The total amount of pheromones that can be stored at any time
pub const PHEROMONE_CAPACITY: f32 = 100.0;
/// The rate at which pheromenes regenerate, per second
pub const PHEROMONE_REGEN_RATE: f32 = 10.0;
/// The rate at which pheromones are spent, per second
pub const PHEROMONE_SPENDING_RATE: f32 = 30.0;

/// The initial mass of spawned structures
pub const STRUCTURE_STARTING_MASS: f32 = 0.5;
/// The mass at which sturctures will despawn
pub const STRUCTURE_DESPAWN_MASS: f32 = 0.01;
/// The rate of growth of structures
pub const STRUCTURE_GROWTH_RATE: f32 = 1.0;
/// The upkeeep cost of each structure
pub const STRUCTURE_UPKEEP_RATE: f32 = 0.1;
