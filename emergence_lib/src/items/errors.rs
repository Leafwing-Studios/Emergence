//! Errors related to items and inventories.

use super::count::ItemCount;

/// Failed to add items to an inventory.
#[derive(Debug, PartialEq, Eq)]
pub struct AddOneItemError {
    /// The number of items that exceed the capacity.
    pub excess_count: usize,
}

/// Failed to add items to an inventory.
#[derive(Debug, PartialEq, Eq)]
pub struct AddManyItemsError {
    /// The number of items that exceeded the capacity.
    pub excess_counts: Vec<ItemCount>,
}

/// Failed to remove items from an item slot.
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveOneItemError {
    /// The number of items that were missing from the inventory.
    pub missing_count: usize,
}

/// Failed to remove many items from an inventory.
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveManyItemsError {
    /// The number of items that were missing from the inventory.
    pub missing_counts: Vec<ItemCount>,
}
