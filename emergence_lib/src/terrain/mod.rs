//! Generating and representing terrain as game objects.
use crate as emergence_lib;
use crate::enum_iter::IterableEnum;
use crate::graphics::Layer;
use crate::graphics::{IntoSprite, LayerRegister};
use bevy::prelude::{Commands, Component, Entity, Res};
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::map::TilemapTileSize;
use bevy_ecs_tilemap::tiles::TilePos;
use emergence_macros::IterableEnum;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
pub use world_query::*;

/// The number of graphics from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;

/// Resource that stores information regarding the size of the game map.
pub struct MapGeometry {
    /// The radius, in graphics, of the map
    pub radius: u32,
    /// The location of the central tile
    pub center: TilePos,
    /// The [`TilemapSize`] of the map
    pub size: TilemapSize,
}

impl MapGeometry {
    /// Computes the total diameter from end-to-end of the game world
    #[inline]
    pub const fn diameter(&self) -> u32 {
        2 * self.radius + 1
    }

    /// Computes the [`TilemapSize`] of the game world
    #[inline]
    pub const fn size(&self) -> TilemapSize {
        self.size
    }

    /// Computes the [`TilePos`] of the tile at the center of this map.
    ///
    /// This is not (0,0) as `bevy_ecs_tilemap` works with `u32` coordinates.
    #[inline]
    pub const fn center(&self) -> TilePos {
        self.center
    }
}

/// The marker component for plain terrain.
#[derive(Component, Clone, Copy)]
pub struct PlainTerrain;

/// The marker component for impassable terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct ImpassableTerrain;

/// The marker component for high terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct HighTerrain;

/// Available terrain types.
#[derive(Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainType {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is impassable.
    Impassable,
    /// Terrain that has higher altitude compared to others.
    High,
}

impl TerrainType {
    /// Creates a tile enttiy corresponding to `self`'s [`TerrainType`] variant
    pub fn create_entity(
        &self,
        commands: &mut Commands,
        position: TilePos,
        layer_register: &Res<LayerRegister>,
    ) -> Entity {
        let mut builder = commands.spawn();

        builder.insert_bundle(self.tile_bundle(position, layer_register));
        match self {
            TerrainType::Plain => {
                builder.insert(PlainTerrain);
            }
            TerrainType::Impassable => {
                builder.insert(ImpassableTerrain);
            }
            TerrainType::High => {
                builder.insert(HighTerrain);
            }
        }
        builder.id()
    }

    /// Choose a random terrain tile based on the given weights
    pub fn choose_random<R: Rng + ?Sized>(
        rng: &mut R,
        weights: &HashMap<TerrainType, f32>,
    ) -> Result<TerrainType, WeightedError> {
        TerrainType::variants()
            .collect::<Vec<TerrainType>>()
            .choose_weighted(rng, |terrain_type| {
                weights.get(terrain_type).copied().unwrap_or_default()
            })
            .copied()
    }
}

impl IntoSprite for TerrainType {
    const ROOT_PATH: &'static str = "terrain";
    const LAYER: Layer = Layer::Terrain;

    fn leaf_path(&self) -> &'static str {
        match self {
            TerrainType::High => "tile-high.png",
            TerrainType::Impassable => "tile-impassable.png",
            TerrainType::Plain => "tile-plain.png",
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
