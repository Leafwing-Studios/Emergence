//! The [`OrganismTilemap`] manages visualization of organisms.
use crate as emergence_lib;
use bevy::prelude::Component;
use bevy_ecs_tilemap::map::TilemapTileSize;
use emergence_macros::IterableEnum;

/// Enumerates organism sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSprite {
    /// Sprite for an Ant
    Ant,
    /// Sprite for a Plant
    Plant,
    /// Sprite for fungi
    Fungi,
}

impl SpriteEnum for OrganismSprite {
    const ROOT_PATH: &'static str = "organisms";
    const TILEMAP: Tilemap = Tilemap::Organisms;

    fn leaf_path(&self) -> &'static str {
        match self {
            OrganismSprite::Ant => "tile-ant.png",
            OrganismSprite::Fungi => "tile-fungus.png",
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
#[derive(Component, Copy, Debug)]
pub struct OrganismsTilemap;

impl TilemapMarker for OrganismsTilemap {
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const MAP_Z: f32 = 1.0;
    type Sprites = OrganismSprite;
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

use crate::graphics::tilemap_marker::TilemapMarker;
use crate::graphics::{SpriteEnum, Tilemap};
pub use world_query::*;
