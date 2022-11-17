//! The [`OrganismTilemap`] manages visualization of organisms.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

pub use world_query::*;

/// Enumerates organism sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSpriteIndex {
    /// Sprite for an Ant
    Ant,
    /// Sprite for a Plant
    Plant,
    /// Sprite for fungi
    Fungi,
}

impl SpriteIndex for OrganismSpriteIndex {
    const ROOT_PATH: &'static str = "organisms";

    fn leaf_path(&self) -> &'static str {
        match self {
            OrganismSpriteIndex::Ant => "tile-ant.png",
            OrganismSpriteIndex::Fungi => "tile-fungus.png",
            OrganismSpriteIndex::Plant => "tile-plant.png",
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
#[derive(Component, Clone, Copy, Debug)]
pub struct OrganismsTilemap;

impl TilemapMarker for OrganismsTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 1.0;
    type Index = OrganismSpriteIndex;
}

/// We are forced to make this a module for now, in order to apply `#[allow(missing_docs)]`, as
/// `WorldQuery` generates structs that triggers `#[deny(missing_docs)]`. As this issue is fixed in
/// Bevy 0.9,  this module can be flattened once this crate and [`bevy_ecs_tilemap`] support 0.9.
#[allow(missing_docs)]
mod world_query {
    use crate::graphics::organisms::OrganismsTilemap;
    use bevy::ecs::query::WorldQuery;
    use bevy::prelude::With;
    use bevy_ecs_tilemap::prelude::TileStorage;
    //Fixed in bevy 0.9.1: https://github.com/bevyengine/bevy/issues/6593
    use bevy::ecs::entity::Entity;

    /// A query item (implements [`WorldQuery`]) specifying a search for `TileStorage` associated with a
    /// `Tilemap` that has the `OrganismTilemap` component type.
    #[derive(WorldQuery)]
    pub struct OrganismStorage<'a> {
        /// Query for tile storage.
        pub storage: &'a TileStorage,
        /// Only query for those entities that contain the relevant tilemap type.
        _organism_tile_map: With<OrganismsTilemap>,
    }
}
