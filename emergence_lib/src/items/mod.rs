//! Everything related to items and crafting.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::Manifest;

pub(crate) mod errors;
pub(crate) mod inventory;
pub(crate) mod recipe;
pub(crate) mod slot;

/// The unique identifier of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId {
    /// The associated string.
    id: &'static str,
}

// TODO: these should be loaded from file
impl ItemId {
    /// The item ID of an Acacia leaf.
    pub fn acacia_leaf() -> Self {
        Self { id: "acacia_leaf" }
    }

    /// The item ID of a Leuco chunk.
    pub fn leuco_chunk() -> Self {
        Self { id: "leuco_chunk" }
    }

    /// The item ID of an ant egg.
    pub fn ant_egg() -> Self {
        Self { id: "ant_egg" }
    }

    /// An item ID solely used for testing.
    #[cfg(test)]
    pub fn test() -> Self {
        Self { id: "test" }
    }
}

impl Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// The data associated with each item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The number of items that can fit in a single item slot.
    stack_size: usize,
}

impl ItemData {
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
}

/// The data definitions for all items.
pub(crate) type ItemManifest = Manifest<ItemId, ItemData>;

/// A specific amount of a given item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCount {
    /// The unique identifier of the item being counted.
    item_id: ItemId,

    /// The number of items.
    count: usize,
}

impl ItemCount {
    /// Create a new item count with the given number of items.
    pub fn new(item_id: ItemId, count: usize) -> Self {
        Self { item_id, count }
    }

    /// A single one of the given item.
    pub fn one(item_id: ItemId) -> Self {
        Self { item_id, count: 1 }
    }

    /// The unique identifier of the item being counted.
    pub fn item_id(&self) -> ItemId {
        self.item_id
    }

    /// The number of items.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Display for ItemCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.item_id, self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_item_type_and_count() {
        let item_count = ItemCount::new(ItemId::acacia_leaf(), 3);

        assert_eq!(format!("{item_count}"), "acacia_leaf (3)".to_string());
    }
}
