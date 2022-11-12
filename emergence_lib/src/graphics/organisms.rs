//! The [`OrganismTilemap`] manages visualization of organisms.
use crate as emergence_lib;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates organisms
#[derive(Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSprite {
    /// An ant
    Ant,
    /// A fungi
    Fungi,
    /// A plant
    Plant,
}

impl IntoSprite for OrganismSprite {
    const ROOT_PATH: &'static str = "organisms";
    const LAYER: Layer = Layer::Organisms;

    fn leaf_path(&self) -> &'static str {
        match self {
            OrganismSprite::Ant => "tile-ant.png",
            OrganismSprite::Fungi => "tile-fungi.png",
            OrganismSprite::Plant => "tile-plant.png",
        }
    }
}

/// Marker component for entity that manages visualization of organisms.
///
/// The organism tilemap lies on top of the [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap), and
/// keeps track of visualizations of organisms at terrain locations. It is congruent to
/// [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap) in grid size and tile size (for now). Later,
/// we might find it useful to use a different tile size, but the grid size will always remain the
/// same as that of [`TerrainTilemap`](crate::graphics::terrain::TerrainTilemap).
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
    use crate::graphics::organisms::OrganismTilemap;
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

use crate::graphics::{IntoSprite, Layer};
pub use world_query::*;
