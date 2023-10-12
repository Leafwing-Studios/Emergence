//! Defines write-only data for each variety of unit.

use bevy::{
    reflect::{Reflect, TypePath, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::loader::IsRawManifest,
    organisms::{OrganismVariety, RawOrganismVariety},
    simulation::time::Days,
    units::{basic_needs::Diet, WanderingBehavior},
};

use super::{basic_needs::RawDiet, Manifest};

/// The marker type for [`Id<Unit>`](super::Id).
#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
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
    /// How long can this unit go without eating before it dies?
    pub max_age: Days,
    /// How many actions will units of this type take while wandering before picking a new goal?
    ///
    /// This stores a [`WeightedIndex`](rand::distributions::WeightedIndex) to allow for multimodal distributions.
    pub wandering_behavior: WanderingBehavior,
}

impl UnitData {
    /// Constructs a new [`UnitData`] from the given [`OrganismVariety`] and [`Diet`].
    #[cfg(test)]
    pub fn simple(name: &str, diet: Diet) -> Self {
        Self {
            organism_variety: OrganismVariety::simple(name),
            diet,
            max_impatience: 10,
            max_age: Days(10.0),
            wandering_behavior: WanderingBehavior::default(),
        }
    }
}

/// The unprocessed equivalent of [`UnitData`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawUnitData {
    /// The data shared by all organisms
    pub organism_variety: RawOrganismVariety,
    /// What this unit type needs to eat
    pub diet: RawDiet,
    /// How much impatience this unit can accumulate before getting too frustrated and picking a new task.
    pub max_impatience: u8,
    /// How long can this unit go without eating before it dies?
    pub max_age: f32,
    /// How many actions will units of this type take while wandering before picking a new goal?
    ///
    /// This stores a [`WeightedIndex`](rand::distributions::WeightedIndex) to allow for multimodal distributions.
    pub wandering_behavior: WanderingBehavior,
}

impl From<RawUnitData> for UnitData {
    fn from(raw: RawUnitData) -> Self {
        assert!(
            raw.max_age > 0.0,
            "Unit max age must be positive (got {})",
            raw.max_age
        );

        Self {
            organism_variety: raw.organism_variety.into(),
            diet: raw.diet.into(),
            max_impatience: raw.max_impatience,
            max_age: Days(raw.max_age),
            wandering_behavior: raw.wandering_behavior,
        }
    }
}

/// The [`UnitManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, TypePath, PartialEq)]
#[uuid = "c8f6e1a1-20a0-4629-8df1-2e1fa313fcb9"]
pub struct RawUnitManifest {
    /// The data for each item.
    pub unit_types: HashMap<String, RawUnitData>,
}

impl IsRawManifest for RawUnitManifest {
    const EXTENSION: &'static str = "unit_manifest.json";

    type Marker = Unit;
    type Data = UnitData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.unit_types.clone() {
            let data = raw_data.into();

            // No additional preprocessing is needed.
            manifest.insert(raw_id, data)
        }

        manifest
    }
}
