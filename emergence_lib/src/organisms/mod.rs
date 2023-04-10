//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    simulation::SimulationSet,
    structures::structure_manifest::{Structure, StructureManifest},
    units::unit_manifest::{Unit, UnitManifest},
};

use self::{
    energy::{
        consume_energy, kill_organisms_when_out_of_energy, tick_vigor_modifiers, EnergyPool,
        VigorModifier,
    },
    lifecycle::{sprout_seeds, transform_when_lifecycle_complete, Lifecycle, RawLifecycle},
};

pub mod energy;
pub mod lifecycle;

/// The [`Id`] of an organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrganismId {
    /// Represents a [`Structure`].
    Structure(Id<Structure>),
    /// Represents a [`Unit`].
    Unit(Id<Unit>),
}

/// The unprocessed equivalent of [`OrganismId`].
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RawOrganismId {
    /// Represents a [`Structure`].
    Structure(String),
    /// Represents a [`Unit`].
    Unit(String),
}

impl RawOrganismId {
    /// Creates a new unit-based [`RawOrganismId`] from a string.
    pub fn unit(name: &str) -> RawOrganismId {
        RawOrganismId::Unit(name.to_string())
    }

    /// Creates a new structure-based [`RawOrganismId`] from a string.
    pub fn structure(name: &str) -> RawOrganismId {
        RawOrganismId::Structure(name.to_string())
    }
}

impl From<RawOrganismId> for OrganismId {
    fn from(raw_organism_id: RawOrganismId) -> Self {
        match raw_organism_id {
            RawOrganismId::Structure(raw_structure_id) => {
                OrganismId::Structure(Id::from_name(raw_structure_id))
            }
            RawOrganismId::Unit(raw_unit_id) => OrganismId::Unit(Id::from_name(raw_unit_id)),
        }
    }
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
    /// The modifier to both working speed and energy drain rate.
    vigor_modfiier: VigorModifier,
    /// The ways this organism can transform, and the progress toward doing so.
    lifecycle: Lifecycle,
}

impl OrganismBundle {
    /// Create a new [`OrganismBundle`]
    pub(crate) fn new(energy_pool: EnergyPool, lifecycle: Lifecycle) -> OrganismBundle {
        OrganismBundle {
            organism: Organism,
            energy_pool,
            vigor_modfiier: VigorModifier::None,
            lifecycle,
        }
    }
}

/// Information about a variety of organism.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganismVariety {
    /// The "base" form that we should display to players in menus and for ghosts?
    pub prototypical_form: OrganismId,
    /// The lifecycle of this organism, which reflect how and why it can change form.
    pub lifecycle: Lifecycle,
    /// Controls the maximum energy, and the rate at which it drains.
    pub energy_pool: EnergyPool,
}

/// The unprocessed form of an [`OrganismVariety`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawOrganismVariety {
    /// The "base" form that we should display to players in menus and for ghosts?
    pub prototypical_form: RawOrganismId,
    /// The lifecycle of this organism, which reflect how and why it can change form.
    pub lifecycle: RawLifecycle,
    /// Controls the maximum energy, and the rate at which it drains.
    pub energy_pool: EnergyPool,
}

impl From<RawOrganismVariety> for OrganismVariety {
    fn from(raw: RawOrganismVariety) -> Self {
        OrganismVariety {
            prototypical_form: raw.prototypical_form.into(),
            lifecycle: raw.lifecycle.into(),
            energy_pool: raw.energy_pool,
        }
    }
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
                consume_energy,
                kill_organisms_when_out_of_energy,
                transform_when_lifecycle_complete,
                sprout_seeds,
                tick_vigor_modifiers,
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
