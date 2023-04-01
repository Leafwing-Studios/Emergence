//! Defines write-only data for each variety of item.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::{loader::RawManifest, Manifest};

/// The marker type for [`Id<Item>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Item;
/// Stores the read-only definitions for all items.
pub(crate) type ItemManifest = Manifest<Item, ItemData>;

/// The data associated with each item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The number of items that can fit in a single item slot.
    stack_size: usize,
}

impl ItemData {
    /// Create new item data.
    #[cfg(test)]
    pub fn new(stack_size: usize) -> Self {
        Self { stack_size }
    }

    /// The number of items that can fit in a single item slot.
    pub fn stack_size(&self) -> usize {
        self.stack_size
    }
}

/// The item manifest as seen in the manifest file.
#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "cd9f4571-b0c4-4641-8d27-1c9c5ad4c812"]
pub(crate) struct RawItemManifest {
    /// The data for each item.
    items: HashMap<String, ItemData>,
}

impl RawManifest for RawItemManifest {
    type Marker = Item;
    type Data = ItemData;

    fn path() -> &'static str {
        "manifests/items.manifest.json"
    }

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (name, raw_data) in &self.items {
            // No additional preprocessing is needed.
            manifest.insert(name, raw_data.clone())
        }

        manifest
    }
}
