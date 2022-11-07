//! Utilities for defining and visualizing game tiles.

use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapGridSize, TilemapId, TilemapType};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileTextureIndex};

use bevy::utils::HashMap;

pub mod organisms;
pub mod position;
pub mod terrain;

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

/// We use a hexagonal map with "pointy-topped" (row oriented) tiles, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// We are using a map with hexagonal tiles.
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// A type that can be transformed into a tile that is compatible with [`bevy_ecs_tilemap`].
pub trait IntoTileBundle {
    /// The corresponding [`TileTextureIndex`] and the [`TilemapId`] layer that it belongs to.
    fn tile_texture(
        &self,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
    ) -> (TilemapId, TileTextureIndex);

    /// The asset path to the [`TileTextureIndex`].
    fn tile_texture_path(&self) -> &'static str;

    /// Uses the data stored in `self` to create a new, matching [`TileBundle`].
    fn as_tile_bundle(
        &self,
        tilemap_id: TilemapId,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
        position: TilePos,
    ) -> TileBundle {
        TileBundle {
            position,
            tilemap_id,
            texture_index: self.tile_texture(tilemap_ids).1,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    Organism,
    Terrain,
}

pub struct LayerLookup {
    map: HashMap<LayerType, TilemapId>,
}
