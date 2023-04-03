//! Defines write-only data for each variety of item.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::{loader::IsRawManifest, Manifest};

/// The marker type for [`Id<Item>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Item;
/// Stores the read-only definitions for all items.
pub type ItemManifest = Manifest<Item, ItemData>;

/// The data associated with each item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The number of items that can fit in a single item slot.
    pub stack_size: usize,
}

/// The [`ItemManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "cd9f4571-b0c4-4641-8d27-1c9c5ad4c812"]
pub struct RawItemManifest {
    /// The data for each item.
    pub items: HashMap<String, ItemData>,
}

impl IsRawManifest for RawItemManifest {
    const EXTENSION: &'static str = "item_manifest.json";

    type Marker = Item;
    type Data = ItemData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in &self.items {
            // No additional preprocessing is needed.
            manifest.insert(raw_id, raw_data.clone())
        }

        manifest
    }
}
