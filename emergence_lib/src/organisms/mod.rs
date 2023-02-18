//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

/// All of the standard components of an [`Organism`]
#[derive(Bundle)]
pub struct OrganismBundle {
    /// The marker component for orgamisms
    pub organism: Organism,
}

impl Default for OrganismBundle {
    fn default() -> Self {
        Self { organism: Organism }
    }
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// Controls the behavior of living organisms
pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, _app: &mut App) {}
}
