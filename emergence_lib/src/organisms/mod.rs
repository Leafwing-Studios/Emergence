//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;
use leafwing_abilities::systems::regenerate_resource_pool;

use self::energy::{kill_organisms_when_out_of_energy, EnergyPool};

pub(crate) mod energy;

/// All of the standard components of an [`Organism`]
#[derive(Bundle)]
pub(crate) struct OrganismBundle {
    /// The marker component for orgamisms
    organism: Organism,
    /// The energy available to this organism
    energy_pool: EnergyPool,
}

impl OrganismBundle {
    /// Create a new [`OrganismBundle`]
    pub(crate) fn new(energy_pool: EnergyPool) -> OrganismBundle {
        OrganismBundle {
            organism: Organism,
            energy_pool,
        }
    }
}

/// Information about a variety of organism.
#[derive(Debug, Clone)]
pub(crate) struct OrganismVariety {
    /// Controls the maximum energy, and the rate at which it drains.
    pub(crate) energy_pool: EnergyPool,
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// Controls the behavior of living organisms
pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(regenerate_resource_pool::<EnergyPool>)
            .add_system(kill_organisms_when_out_of_energy);
    }
}
