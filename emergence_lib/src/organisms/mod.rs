//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use crate::tiles::IntoTileBundle;

use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;

pub mod pathfinding;
pub mod structures;
pub mod units;

/// An [`IndexMap`] of organism images.
pub static ORGANISM_TILE_IMAP: Lazy<IndexMap<OrganismType, &'static str>> = Lazy::new(|| {
    use OrganismType::*;
    indexmap! {
        Ant => "tile-ant.png",
        Fungus => "tile-fungus.png",
        Plant => "tile-plant.png",
    }
});

/// The tile size (hex tile width by hex tile height) in pixels of organism image assets.
pub const ORGANISM_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };

/// The type of [`Organism`]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum OrganismType {
    /// A wandering unit.
    Ant,
    /// A fixed structure that does not photosynthesize.
    Fungus,
    /// A fixed structure that photosynthesizes.
    Plant,
}

impl IntoTileBundle for OrganismType {
    fn tile_texture(&self) -> TileTextureIndex {
        TileTextureIndex(ORGANISM_TILE_IMAP.get_index_of(self).unwrap() as u32)
    }

    fn tile_texture_path(&self) -> &'static str {
        ORGANISM_TILE_IMAP.get(self).unwrap()
    }
}

/// The marker component for all organisms.
#[derive(Component, Clone, Default)]
pub struct Organism;

/// The mass of each element that makes up the entity
#[derive(Component, Clone, Default)]
pub struct Composition {
    /// Mass is represented with an `f32`.
    pub mass: f32,
}

/// An organism is a living component of the game ecosystem.
#[derive(Bundle, Default)]
pub struct OrganismBundle {
    /// Marker component.
    pub organism: Organism,
    /// Defines the elements making up this organism.
    pub composition: Composition,
}

/// Marker component for entities that are part of the organisms tilemap
#[derive(Component)]
pub struct OrganismTilemap;

/// We are forced to make this a module for now, in order to apply `#[allow(missing_docs)]`, as
/// `WorldQuery` generates structs that triggers `#[deny(missing_docs)]`. As this issue is fixed in
/// Bevy 0.9,  this module can be flattened once this crate and [`bevy_ecs_tilemap`] support 0.9.
#[allow(missing_docs)]
mod world_query {
    use crate::organisms::OrganismTilemap;
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
