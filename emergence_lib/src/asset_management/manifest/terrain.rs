//! Data and manifest definitions for terrain.

use bevy::reflect::{FromReflect, Reflect};

use super::Manifest;

/// The marker type for [`Id<Terrain>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Terrain;
/// Stores the read-only definitions for all items.
pub(crate) type TerrainManifest = Manifest<Terrain, TerrainData>;

/// Data stored in a [`TerrainManifest`] for each [`Id<Terrain>`](super::Id).
#[derive(Debug)]
pub(crate) struct TerrainData {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    /// 1.0 is "normal speed".
    walking_speed: f32,
}

impl TerrainData {
    /// Constructs a new [`TerrainData`] object
    pub(crate) fn new(walking_speed: f32) -> Self {
        TerrainData { walking_speed }
    }

    /// Returns the relative walking speed of units on this terrain
    pub(crate) fn walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

impl Default for TerrainManifest {
    // TODO: load this from file
    fn default() -> Self {
        let mut manifest = TerrainManifest::new();

        manifest.insert("rocky", TerrainData::new(2.0));
        manifest.insert("loam", TerrainData::new(1.0));
        manifest.insert("muddy", TerrainData::new(0.5));

        manifest
    }
}
