//! The [`OrganismTilemap`] manages visualization of organisms.

use crate::organisms::OrganismType;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

/// An [`IndexMap`] of organism images.
pub static ORGANISM_TILE_IMAP: Lazy<IndexMap<OrganismType, &'static str>> = Lazy::new(|| {
    use OrganismType::*;
    indexmap! {
        Ant => "tile-ant.png",
        Fungus => "tile-fungus.png",
        Plant => "tile-plant.png",
    }
});

/// Marker component for entity that manages visualization of organisms.
///
/// The organism tilemap lies on top of the [`TerrainTilemap`](crate::terrain::TerrainTilemap), and
/// keeps track of visualizations of organisms at terrain locations. It is congruent to
/// [`TerrainTilemap`](crate::terrain::TerrainTilemap) in grid size and tile size (for now). Later,
/// we might find it useful to use a different tile size, but the grid size will always remain the
/// same as that of [`TerrainTilemap`].
#[derive(Component)]
pub struct OrganismTilemap;

impl OrganismTilemap {
    /// The z-coordinate at which organisms are drawn.
    ///
    /// We want the organism tilemap to be layered on top of the terrain tile map.
    pub const MAP_Z: f32 = 1.0;

    /// The tile size (hex tile width by hex tile height) in pixels of organism image assets.
    pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
}

/// We are forced to make this a module for now, in order to apply `#[allow(missing_docs)]`, as
/// `WorldQuery` generates structs that triggers `#[deny(missing_docs)]`. As this issue is fixed in
/// Bevy 0.9,  this module can be flattened once this crate and [`bevy_ecs_tilemap`] support 0.9.
#[allow(missing_docs)]
mod world_query {
    use crate::tiles::organisms::OrganismTilemap;
    use bevy::ecs::query::WorldQuery;
    use bevy::prelude::With;
    use bevy_ecs_tilemap::prelude::TileStorage;

    /// A query item (implements [`WorldQuery`]) specifying a search for `TileStorage` associated with a
    /// `Tilemap` that has the `OrganismTilemap` component type.
    #[derive(WorldQuery)]
    pub struct OrganismStorage<'a> {
        /// Query for tile storage.
        pub storage: &'a TileStorage,
        /// Only query for those entities that contain the relevant tilemap type.
        _organism_tile_map: With<OrganismTilemap>,
    }
}

pub use world_query::*;
