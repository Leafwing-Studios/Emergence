//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;
use leafwing_abilities::systems::regenerate_resource_pool;

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest},
    simulation::SimulationSet,
    units::unit_manifest::{Unit, UnitManifest},
};

use self::{
    energy::{kill_organisms_when_out_of_energy, EnergyPool},
    lifecycle::{transform_when_lifecycle_complete, Lifecycle},
};

pub(crate) mod energy;
pub(crate) mod lifecycle;

/// The [`Id`] of an organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OrganismId {
    /// Represents a [`Structure`].
    Structure(Id<Structure>),
    /// Represents a [`Unit`].
    Unit(Id<Unit>),
}

impl OrganismId {
    /// Pretty formatting for this type.
    pub(crate) fn display(
        &self,
        structure_manifest: &StructureManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        match self {
            OrganismId::Structure(structure_id) => {
                format!("{} (S)", structure_manifest.name(*structure_id))
            }
            OrganismId::Unit(unit_id) => format!("{} (U)", unit_manifest.name(*unit_id)),
        }
    }
}

/// All of the standard components of an [`Organism`]
#[derive(Bundle)]
pub(crate) struct OrganismBundle {
    /// The marker component for orgamisms
    organism: Organism,
    /// The energy available to this organism
    energy_pool: EnergyPool,
    /// The ways this organism can transform, and the progress toward doing so.
    lifecycle: Lifecycle,
}

impl OrganismBundle {
    /// Create a new [`OrganismBundle`]
    pub(crate) fn new(energy_pool: EnergyPool, lifecycle: Lifecycle) -> OrganismBundle {
        OrganismBundle {
            organism: Organism,
            energy_pool,
            lifecycle,
        }
    }
}

/// Information about a variety of organism.
#[derive(Debug, Clone)]
pub(crate) struct OrganismVariety {
    /// The "base" form that we should display to players in menus and for ghosts?
    pub(crate) prototypical_form: OrganismId,
    /// The lifecycle of this organism, which reflect how and why it can change form.
    pub(crate) lifecycle: Lifecycle,
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
        app.add_systems(
            (
                regenerate_resource_pool::<EnergyPool>,
                kill_organisms_when_out_of_energy,
                transform_when_lifecycle_complete,
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
