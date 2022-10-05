use crate::config::ORGANISM_TILE_IMAP;
use crate::signals::SignalId;
use crate::tiles::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileTexture};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum OrganismType {
    Ant,
    Fungus,
    Plant,
}

impl IntoTile for OrganismType {
    fn tile_texture(&self) -> TileTexture {
        TileTexture((&ORGANISM_TILE_IMAP).get_index_of(self).unwrap() as u32)
    }

    fn tile_texture_path(&self) -> &'static str {
        (&ORGANISM_TILE_IMAP).get(self).unwrap()
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
    pub position: TilePos,
    pub composition: Composition,
    pub signal_id: SignalId,
    pub tile_texture: TileTexture,
}
