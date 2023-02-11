//! Everything related to items and crafting.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::asset_management::manifest::Manifest;

pub mod count;
pub mod errors;
pub mod inventory;
pub mod recipe;
pub mod slot;

/// The unique identifier of an item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(&'static str);

impl ItemId {
    /// The item ID of an Acacia leaf.
    pub fn acacia_leaf() -> Self {
        Self("acacia_leaf")
    }

    /// An item ID solely used for testing.
    #[cfg(test)]
    pub fn test() -> Self {
        Self("test")
    }
}

impl Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
}

/// The data definitions for all items.
pub type ItemManifest = Manifest<ItemId, ItemData>;
