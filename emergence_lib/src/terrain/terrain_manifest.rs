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
pub type TerrainManifest = Manifest<Terrain, TerrainData>;

/// Data stored in a [`TerrainManifest`] for each [`Id<Terrain>`](super::Id).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TerrainData {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    /// 1.0 is "normal speed".
    pub walking_speed: f32,
}

/// The [`TerrainManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "8d6b3b65-9b11-42a9-a795-f95b06653070"]
pub struct RawTerrainManifest {
    /// The data for each item.
    pub terrain_types: HashMap<String, TerrainData>,
}

impl RawManifest for RawTerrainManifest {
    const EXTENSION: &'static str = "terrain_manifest.json";

    type Marker = Terrain;
    type Data = TerrainData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (name, raw_data) in &self.terrain_types {
            // No additional preprocessing is needed.
            manifest.insert(name, raw_data.clone())
        }

        manifest
    }
}
