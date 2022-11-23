//! Trait describing components that mark an entity as something that behaves like a tilemap.

use crate::graphics::sprites::SpriteIndex;
use crate::graphics::MAP_TYPE;
use crate::simulation::generation::GRID_SIZE;
use crate::simulation::map::MapGeometry;
use bevy::asset::AssetServer;
use bevy::log::info;
use bevy::prelude::{Commands, Component, Entity, Res};
use bevy_ecs_tilemap::map::TilemapTileSize;
use bevy_ecs_tilemap::prelude::get_tilemap_center_transform;
use bevy_ecs_tilemap::TilemapBundle;
use std::fmt::Debug;

/// Trait describing components that mark an entity as something that behaves like a tilemap.
pub trait TilemapMarker: Copy + Component + Debug {
    /// The tile size (hex tile width by hex tile height) in pixels of the tilemap's tile image assets.
    const TILE_SIZE: TilemapTileSize;
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
        commands
            .spawn(TilemapBundle {
                grid_size: GRID_SIZE,
                map_type: MAP_TYPE,
                size: map_geometry.size(),
                texture,
                tile_size: Self::TILE_SIZE,
                transform: get_tilemap_center_transform(
                    &map_geometry.size(),
                    &GRID_SIZE,
                    &MAP_TYPE,
                    Self::MAP_Z,
                ),
                ..Default::default()
            })
            .insert(*self)
            .id()
    }
}
