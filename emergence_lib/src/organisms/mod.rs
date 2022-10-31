use crate::tiles::IntoTileBundle;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ecs_tilemap::tiles::TileTexture;
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
    Ant,
    Fungus,
    Plant,
}

impl IntoTileBundle for OrganismType {
    fn tile_texture(&self) -> TileTexture {
        TileTexture(ORGANISM_TILE_IMAP.get_index_of(self).unwrap() as u32)
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
    pub mass: f32,
}

#[derive(Bundle, Default)]
pub struct OrganismBundle {
    pub organism: Organism,
    pub composition: Composition,
}

/// Marker component for entities that are part of the organisms tilemap
#[derive(Component)]
pub struct OrganismTilemap;

/// A query item (implements [`WorldQuery`]) specifying a search for `TileStorage` associated with a
/// `Tilemap` that has the `OrganismTilemap` component type.
#[derive(WorldQuery)]
pub struct OrganismStorage<'a> {
    pub storage: &'a TileStorage,
    _organism_tile_map: With<OrganismTilemap>,
}

// #[derive(Deref)]
// pub struct OrganismTileStorage<'t>(&'t TileStorage);
//
// impl<'t, 'w: 't, 's: 't> From<&'_ Query<'w, 's, &'_ TileStorage, With<OrganismTilemap>>>
//     for OrganismTileStorage<'t>
// {
//     fn from(value: &'_ Query<'w, 's, &'_ TileStorage, With<OrganismTilemap>>) -> Self {
//         OrganismTileStorage(value.single())
//     }
// }
