//! Defines write-only data for each variety of unit.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::{loader::RawManifest, RawId},
    organisms::OrganismVariety,
    units::{hunger::Diet, WanderingBehavior},
};

use super::Manifest;

/// The marker type for [`Id<Unit>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Unit;
/// Stores the read-only definitions for all units.
pub type UnitManifest = Manifest<Unit, UnitData>;

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

/// The [`UnitManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "c8f6e1a1-20a0-4629-8df1-2e1fa313fcb9"]
pub struct RawUnitManifest {
    /// The data for each item.
    pub unit_types: HashMap<RawId<Unit>, UnitData>,
}

impl RawManifest for RawUnitManifest {
    const EXTENSION: &'static str = "unit_manifest.json";

    type Marker = Unit;
    type Data = UnitData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in &self.unit_types {
            // No additional preprocessing is needed.
            manifest.insert(raw_id.name(), raw_data.clone())
        }

        manifest
    }
}
