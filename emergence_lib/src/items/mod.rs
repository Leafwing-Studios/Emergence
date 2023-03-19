//! Everything related to items and crafting.

use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::{Id, Item, ItemManifest};

pub(crate) mod errors;
pub(crate) mod inventory;
pub(crate) mod recipe;
pub(crate) mod slot;

/// The data associated with each item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The number of items that can fit in a single item slot.
    stack_size: usize,
}

impl ItemData {
    /// Create new item data.
    pub fn new(stack_size: usize) -> Self {
        Self { stack_size }
    }

    /// The number of items that can fit in a single item slot.
    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    // TODO: Remove this once we can load item data from asset files
    /// A leaf from an acacia plant.
    pub fn acacia_leaf() -> Self {
        Self { stack_size: 10 }
    }

    // TODO: Remove this once we can load item data from asset files
    /// A piece of a leuco mushroom.
    pub fn leuco_chunk() -> Self {
        Self { stack_size: 5 }
    }

    // TODO: Remove this once we can load item data from asset files
    /// An egg that will hatch into a grown ant.
    pub fn ant_egg() -> Self {
        Self { stack_size: 5 }
    }
}

/// A specific amount of a given item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ItemCount {
    /// The unique identifier of the item being counted.
    item_id: Id<Item>,

    /// The number of items.
    count: usize,
}

impl ItemCount {
    /// Create a new item count with the given number of items.
    pub(crate) fn new(item_id: Id<Item>, count: usize) -> Self {
        Self { item_id, count }
    }

    /// A single one of the given item.
    pub(crate) fn one(item_id: Id<Item>) -> Self {
        Self { item_id, count: 1 }
    }

    /// The unique identifier of the item being counted.
    pub fn item_id(&self) -> Id<Item> {
        self.item_id
    }

    /// The number of items.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The pretty text formatting of this type.
    pub fn display(&self, item_manifest: &ItemManifest) -> String {
        let name = item_manifest.name(self.item_id);
        format!("{}, ({})", name, self.count)
    }
}
