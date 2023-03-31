use bevy::reflect::{FromReflect, Reflect};
use leafwing_abilities::prelude::Pool;

use crate::{
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::Lifecycle,
        OrganismId, OrganismVariety,
    },
    units::{hunger::Diet, WanderingBehavior},
};

use super::{Id, Manifest};

/// The marker type for [`Id<Unit>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Unit;
/// Stores the read-only definitions for all units.
pub(crate) type UnitManifest = Manifest<Unit, UnitData>;

/// The data associated with each variety of unit
#[derive(Debug, Clone)]
pub(crate) struct UnitData {
    /// The data shared by all organisms
    pub(super) organism_variety: OrganismVariety,
    /// What this unit type needs to eat
    pub(super) diet: Diet,
    /// How much impatience this unit can accumulate before getting too frustrated and picking a new task.
    pub(super) max_impatience: u8,
    /// How many actions will units of this type take while wandering before picking a new goal?
    ///
    /// This stores a [`WeightedIndex`] to allow for multimodal distributions.
    pub(super) wandering_behavior: WanderingBehavior,
}

impl UnitData {
    /// Returns the [`OrganismVariety`] data for this type of unit.
    pub(crate) fn organism_variety(&self) -> &OrganismVariety {
        &self.organism_variety
    }

    /// Returns the [`Diet`] for this type of unit.
    pub(crate) fn diet(&self) -> &Diet {
        &self.diet
    }
}

impl Default for UnitManifest {
    fn default() -> Self {
        let mut manifest: UnitManifest = Manifest::new();

        // TODO: load this from disk
        manifest.insert(
            "ant",
            UnitData {
                organism_variety: OrganismVariety {
                    prototypical_form: OrganismId::Unit(Id::from_name("ant")),
                    lifecycle: Lifecycle::STATIC,
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                },
                diet: Diet::new(Id::from_name("leuco_chunk"), Energy(50.)),
                max_impatience: 10,
                wandering_behavior: WanderingBehavior::from_iter([(1, 0.7), (8, 0.2), (16, 0.1)]),
            },
        );

        manifest
    }
}
