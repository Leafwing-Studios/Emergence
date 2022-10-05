use crate::organisms::OrganismType;
use crate::terrain::TerrainType;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapGridSize, TilemapSize, TilemapTileSize};
use bevy_ecs_tilemap::tiles::TilePos;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;

// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

pub static ORGANISM_TILE_IMAP: Lazy<IndexMap<OrganismType, &'static str>> = Lazy::new(|| {
    use OrganismType::*;
    indexmap! {
        Ant => "tile-ant.png",
        Fungus => "tile-fungus.png",
        Plant => "tile-plant.png",
    }
});
pub const ORGANISM_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
// We want the organism tilemap to be layered on top of the terrain tile map.
pub const ORGANISM_TILEMAP_Z: f32 = 1.0;

pub static TERRAIN_TILE_IMAP: Lazy<IndexMap<TerrainType, &'static str>> = Lazy::new(|| {
    use TerrainType::*;
    indexmap! {
        High => "tile-high.png",
        Impassable => "tile-impassable.png",
        Plain => "tile-plain.png",
    }
});
pub const TERRAIN_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
pub const TERRAIN_TILEMAP_Z: f32 = 0.0;

pub const TILE_BUFFER: f32 = 0.0;

pub const MAP_RADIUS: u32 = 10;
pub const MAP_DIAMETER: u32 = 2 * MAP_RADIUS + 1;
pub const MAP_SIZE: TilemapSize = TilemapSize {
    x: MAP_DIAMETER,
    y: MAP_DIAMETER,
};
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
pub const MAP_CENTER: TilePos = TilePos {
    x: MAP_RADIUS + 1,
    y: MAP_RADIUS + 1,
};

pub const N_ANT: usize = 5;
pub const N_PLANT: usize = 10;
pub const N_FUNGI: usize = 10;

pub const PHEROMONE_CAPACITY: f32 = 100.0;
pub const PHEROMONE_REGEN_RATE: f32 = 10.0;
pub const PHEROMONE_SPENDING_RATE: f32 = 30.0;

pub const STRUCTURE_STARTING_MASS: f32 = 0.5;
pub const STRUCTURE_DESPAWN_MASS: f32 = 0.01;
pub const STRUCTURE_GROWTH_RATE: f32 = 1.0;
pub const STRUCTURE_UPKEEP_RATE: f32 = 0.1;
