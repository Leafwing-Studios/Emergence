use bevy::prelude::*;

use crate::{id::ID, position::Position};

/// The marker component for all organisms
#[derive(Clone, Default)]
pub struct Organism;

/// Denotes impassable terrain
#[derive(Clone, Default)]
pub struct Impassable;

/// The mass of each element that makes up the entity
#[derive(Clone, Default)]
pub struct Composition {
    pub mass: f32,
}

#[derive(Bundle, Default)]
pub struct OrganismBundle {
    pub organism: Organism,
    pub position: Position,
    pub impassable: Impassable,
    pub composition: Composition,
    pub id: ID,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}
