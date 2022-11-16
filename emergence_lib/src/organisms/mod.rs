//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use crate::organisms::structures::StructureType;
use crate::organisms::units::UnitType;
use bevy::prelude::*;

pub mod structures;
pub mod units;

/// Available types of organisms
#[derive(Component, Clone)]
pub enum OrganismType {
    Unit { inner: UnitType },
    Structure { inner: StructureType },
}

/// The mass of each element that makes up the entity
#[derive(Component, Clone, Default)]
pub struct Composition {
    /// Mass is represented with an `f32`.
    pub mass: f32,
}

/// An organism is a living component of the game ecosystem.
#[derive(Bundle)]
pub struct OrganismBundle {
    /// Data describing the type of the organism
    pub organism: OrganismType,
    /// Defines the elements making up this organism.
    pub composition: Composition,
}
