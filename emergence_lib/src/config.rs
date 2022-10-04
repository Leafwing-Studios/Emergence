use crate::organisms::OrganismType;
use crate::terrain::TerrainType;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapGridSize, TilemapSize, TilemapTileSize};
use bevy_ecs_tilemap::tiles::TilePos;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;

pub static TERRAIN_TILE_IMAP: Lazy<IndexMap<TerrainType, &'static str>> = Lazy::new(|| {
    indexmap! {
        TerrainType::Plain => "tile.png",
        TerrainType::High => "tile-high.png",
        TerrainType::Impassable => "tile-impassable.png",
    }
});
pub const TERRAIN_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
pub const TERRAIN_GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

pub static ORGANISM_TILE_IMAP: Lazy<IndexMap<OrganismType, &'static str>> = Lazy::new(|| {
    indexmap! {
        OrganismType::Ant => "tile-ant.png",
        OrganismType::Fungus => "tile-fungus.png",
        OrganismType::Plant => "tile-plant.png",
    }
});
pub const ORGANISM_TILE_SIZE: TilemapTileSize = TERRAIN_TILE_SIZE;
pub const ORGANISM_GRID_SIZE: TilemapGridSize = TERRAIN_GRID_SIZE;

// pub const TILE_PNG: &'static str = "tile.png";
// pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 59.0, y: 59.0 };
// pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 59.0, y: 59.0 };

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
pub const STRUCTURE_DESPAWN_MASS: f32 = 0.1;
pub const STRUCTURE_GROWTH_RATE: f32 = 1.0;
pub const STRUCTURE_UPKEEP_RATE: f32 = 1.0;
