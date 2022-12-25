//! Trait describing components that mark an entity as something that behaves like a tilemap.

use crate::graphics::sprites::SpriteIndex;
use crate::graphics::MAP_TYPE;
use crate::simulation::map::MapGeometry;
use bevy::asset::AssetServer;
use bevy::log::info;
use bevy::prelude::{Commands, Component, Entity, Res};
use bevy_ecs_tilemap::map::TilemapTileSize;
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, TilemapGridSize};
use bevy_ecs_tilemap::TilemapBundle;
use std::fmt::Debug;

/// Trait describing components that mark an entity as something that behaves like a tilemap.
pub trait TilemapLike: Copy + Component + Debug {
    /// The tile size (hex tile width by hex tile height) in pixels of the tilemap's tile image assets.
    ///
    /// Note that in general, a regular hexagon "pointy top" (row oriented) hexagon has a
    /// `width:height` ratio of [`sqrt(3.0)/2.0`](https://www.redblobgames.com/grids/hexagons/#basics),
    /// but because pixels are integer values, there will usually be some rounding, so tiles will
    /// only approximately match this ratio.
    const TILE_SIZE: TilemapTileSize;
    /// The grid size in pixels. If it is `None`, it will default to be the same as [`TILE_SIZE`].
    ///
    /// This can differ from [`TILE_SIZE`], in order to overlap tiles, or pad tiles.
    const GRID_SIZE: Option<TilemapGridSize>;
    /// The z-coordinate at which graphics for this tilemap-like are drawn.
    const MAP_Z: f32;
    /// The sprite index associated with this tilemap
    type Index: SpriteIndex;

    /// Spawn a corresponding `bevy_ecs_tilemap` [`TilemapBundle`]
    fn spawn(
        &self,
        commands: &mut Commands,
        map_geometry: &Res<MapGeometry>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        let texture = Self::Index::load(asset_server);

        info!("Inserting TilemapBundle for {:?}...", self);

        let grid_size = if let Some(grid_size) = Self::GRID_SIZE {
            grid_size
        } else {
            Self::TILE_SIZE.into()
        };

        commands
            .spawn(TilemapBundle {
                grid_size,
                map_type: MAP_TYPE,
                size: map_geometry.size(),
                texture,
                tile_size: Self::TILE_SIZE,
                transform: get_tilemap_center_transform(
                    &map_geometry.size(),
                    &grid_size,
                    &MAP_TYPE,
                    Self::MAP_Z,
                ),
                ..Default::default()
            })
            .insert(*self)
            .id()
    }
}
