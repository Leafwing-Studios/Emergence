//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

pub mod structures;
pub mod units;

/// The mass of each element that makes up the entity
#[derive(Component, Clone, Default)]
pub struct Composition {
    /// Mass is represented with an `f32`.
    pub mass: f32,
}

/// An organism is a living component of the game ecosystem.
#[derive(Bundle)]
pub struct OrganismBundle {
    /// Defines the elements making up this organism.
    pub composition: Composition,
}
