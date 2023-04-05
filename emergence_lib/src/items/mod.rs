//! Everything related to items and crafting.

use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::Id;

use self::item_manifest::{Item, ItemManifest};

pub mod errors;
pub mod inventory;
pub mod item_manifest;
pub mod slot;

/// A specific amount of a given item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemCount {
    /// The unique identifier of the item being counted.
    pub item_id: Id<Item>,

    /// The number of items.
    pub count: u32,
}

impl ItemCount {
    /// Create a new item count with the given number of items.
    pub fn new(item_id: Id<Item>, count: u32) -> Self {
        Self { item_id, count }
    }

    /// A single one of the given item.
    pub fn one(item_id: Id<Item>) -> Self {
        Self { item_id, count: 1 }
    }

    /// The pretty text formatting of this type.
    pub fn display(&self, item_manifest: &ItemManifest) -> String {
        let name = item_manifest.name(self.item_id);
        format!("{}, ({})", name, self.count)
    }
}
