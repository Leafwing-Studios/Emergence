use crate::signals::SignalId;
use crate::terrain::ImpassableTerrain;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileTexture};

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
    pub impassable: ImpassableTerrain,
    pub composition: Composition,
    pub id: SignalId,
    pub tile_texture: TileTexture,
}
