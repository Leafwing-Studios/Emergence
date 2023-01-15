//! Everything related to items and crafting.

use std::fmt::Display;

pub mod count;
pub mod errors;
pub mod inventory;
pub mod recipe;
pub mod slot;

/// The unique identifier of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemId(&'static str);

impl ItemId {
    /// The item ID of an Acacia leaf.
    pub fn acacia_leaf() -> Self {
        Self("acacia_leaf")
    }
}

impl Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
