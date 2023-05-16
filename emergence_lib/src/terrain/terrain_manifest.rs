//! Defines write-only data for each variety of terrain.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::{loader::IsRawManifest, Manifest};

/// The marker type for [`Id<Terrain>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Terrain;
/// Stores the read-only definitions for all items.
pub type TerrainManifest = Manifest<Terrain, TerrainData>;

/// Data stored in a [`TerrainManifest`] for each [`Id<Terrain>`](super::Id).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TerrainData {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    /// 1.0 is "normal speed".
    ///
    /// Note that this only affects the walking speed of units that are not on a path.
    pub walking_speed: f32,
    /// The amount of water that can be stored in one volume of this terrain type.
    ///
    /// This is relative to empty space, which has a capacity of 1.0.
    /// Generally this value should be between 0.05 and 0.5.
    pub water_capacity: f32,
    /// The relative rate at which water flows through this terrain type.
    ///
    /// This is relative to empty space, which has a flow rate of 1.0.
    /// Generally this value should be between 0.05 and 0.3.
    pub water_flow_rate: f32,
    /// The evaporation rate of water from this terrain type.
    ///
    /// This is relative to empty space, which has an evaporation rate of 1.0.
    /// Generally this value should be between 0.05 and 0.5.
    pub water_evaporation_rate: f32,
}

impl Default for TerrainData {
    fn default() -> Self {
        Self {
            walking_speed: 1.0,
            water_capacity: 0.2,
            water_flow_rate: 0.1,
            water_evaporation_rate: 0.1,
        }
    }
}

/// The [`TerrainManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "8d6b3b65-9b11-42a9-a795-f95b06653070"]
pub struct RawTerrainManifest {
    /// The data for each item.
    pub terrain_types: HashMap<String, TerrainData>,
}

impl IsRawManifest for RawTerrainManifest {
    const EXTENSION: &'static str = "terrain_manifest.json";

    type Marker = Terrain;
    type Data = TerrainData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.terrain_types.clone() {
            // No additional preprocessing is needed.
            manifest.insert(raw_id, raw_data)
        }

        manifest
    }
}
