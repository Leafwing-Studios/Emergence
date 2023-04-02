//! Defines write-only data for each variety of unit.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use leafwing_abilities::prelude::Pool;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::loader::RawManifest,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitData {
    /// The data shared by all organisms
    pub organism_variety: OrganismVariety,
    /// What this unit type needs to eat
    pub diet: Diet,
    /// How much impatience this unit can accumulate before getting too frustrated and picking a new task.
    pub max_impatience: u8,
    /// How many actions will units of this type take while wandering before picking a new goal?
    ///
    /// This stores a [`WeightedIndex`](rand::distributions::WeightedIndex) to allow for multimodal distributions.
    pub wandering_behavior: WanderingBehavior,
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

/// The [`UnitManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "c8f6e1a1-20a0-4629-8df1-2e1fa313fcb9"]
pub struct RawUnitManifest {
    /// The data for each item.
    pub unit_types: HashMap<String, UnitData>,
}

impl RawManifest for RawUnitManifest {
    const EXTENSION: &'static str = "terrain_manifest.json";

    type Marker = Unit;
    type Data = UnitData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (name, raw_data) in &self.unit_types {
            // No additional preprocessing is needed.
            manifest.insert(name, raw_data.clone())
        }

        manifest
    }
}
