//! The [`ProduceTilemap`] manages visualization of produce.
use crate as emergence_lib;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::ecs::component::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates produce sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum ProduceSprite {}

/// Marker component for entity that manages visualization of produce.
///
/// See also:
/// *
/// * [`OrganismTilemap`](crate::graphics::organisms::OrganismTilemap), which lies on top of the
/// terrain tilemap, and manages visualization of organisms
#[derive(Component, Debug, Copy)]
pub struct ProduceTilemap;

impl TilemapMarker for ProduceTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 2.0;
    type Sprites = ProduceSprite;
}

pub use world_query::*;

/// We are forced to make this a module for now, in order to apply `#[allow(missing_docs)]`, as
/// `WorldQuery` generates structs that triggers `#[deny(missing_docs)]`. As this issue is fixed in
/// Bevy 0.9,  this module can be flattened once this crate and [`bevy_ecs_tilemap`] support 0.9.
#[allow(missing_docs)]
pub mod world_query {
    use crate::graphics::produce::ProduceTilemap;
    //Fixed in bevy 0.9.1: https://github.com/bevyengine/bevy/issues/6593
    use bevy::ecs::entity::Entity;
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
        _terrain_tile_map: With<ProduceTilemap>,
    }
}
