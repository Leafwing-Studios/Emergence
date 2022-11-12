//! The [`TerrainTilemap`] manages visualization of terrain.

use crate as emergence_lib;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates terrain types
#[derive(Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainSprite {
    /// High
    High,
    /// Impassable
    Impassable,
    /// Plains
    Plain,
}

impl IntoSprite for TerrainSprite {
    const ROOT_PATH: &'static str = "terrain";
    const LAYER: Layer = Layer::Terrain;

    fn leaf_path(&self) -> &'static str {
        match self {
            TerrainSprite::High => "tile-high.png",
            TerrainSprite::Impassable => "tile-impassable.png",
            TerrainSprite::Plain => "tile-plain.png",
        }
    }
}

/// Marker component for entity that manages visualization of terrain.
///
/// See also, the [`OrganismTilemap`](crate::graphics::organisms::OrganismTilemap), which lies on top of the
/// terrain tilemap, and manages visualization of organisms.
#[derive(Component)]
pub struct TerrainTilemap;

impl TerrainTilemap {
    /// The tile size (hex tile width by hex tile height) in pixels of tile image assets.
    pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    /// The z-coordinate at which graphics are drawn.
    pub const MAP_Z: f32 = 0.0;
}

/// We are forced to make this a module for now, in order to apply `#[allow(missing_docs)]`, as
/// `WorldQuery` generates structs that triggers `#[deny(missing_docs)]`. As this issue is fixed in
/// Bevy 0.9,  this module can be flattened once this crate and [`bevy_ecs_tilemap`] support 0.9.
#[allow(missing_docs)]
pub mod world_query {
    use crate::graphics::terrain::TerrainTilemap;
    use bevy::ecs::query::WorldQuery;
    use bevy::prelude::With;
    use bevy_ecs_tilemap::prelude::TileStorage;

    /// A [`WorldQuery`] specifying a search for `TileStorage` associated with a
    /// `Tilemap` that has the `TerrainTilemap` component type.
    #[derive(WorldQuery)]
    pub struct TerrainStorage<'a> {
        /// Queries for tile storage.
        pub storage: &'a TileStorage,
        /// Only query for those entities that contain the relevant tilemap type.
        _terrain_tile_map: With<TerrainTilemap>,
    }
}

use crate::graphics::{IntoSprite, Layer};
pub use world_query::*;
