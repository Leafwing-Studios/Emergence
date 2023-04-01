//! Defines write-only data for each variety of terrain.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::{loader::RawManifest, Manifest};

/// The marker type for [`Id<Terrain>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Terrain;
/// Stores the read-only definitions for all items.
pub(crate) type TerrainManifest = Manifest<Terrain, TerrainData>;

/// Data stored in a [`TerrainManifest`] for each [`Id<Terrain>`](super::Id).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TerrainData {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    /// 1.0 is "normal speed".
    walking_speed: f32,
}

impl TerrainData {
    /// Returns the relative walking speed of units on this terrain
    pub(crate) fn walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

/// The [`TerrainManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "8d6b3b65-9b11-42a9-a795-f95b06653070"]
pub(super) struct RawTerrainManifest {
    /// The data for each item.
    terrain_types: HashMap<String, TerrainData>,
}

impl RawManifest for RawTerrainManifest {
    type Marker = Terrain;
    type Data = TerrainData;

    fn path() -> &'static str {
        "manifests/terrain.manifest.json"
    }

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (name, raw_data) in &self.terrain_types {
            // No additional preprocessing is needed.
            manifest.insert(name, raw_data.clone())
        }

        manifest
    }
}