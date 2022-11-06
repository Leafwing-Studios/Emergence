//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use crate::tiles::IntoTileBundle;

use crate::tiles::organisms::ORGANISM_TILE_IMAP;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;

pub mod structures;
pub mod units;

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
