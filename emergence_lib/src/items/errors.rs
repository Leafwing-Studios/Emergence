//! Errors related to items and inventories.

use super::ItemCount;

/// Failed to add items to an inventory.
#[derive(Debug, PartialEq, Eq)]
pub struct AddOneItemError {
    /// The number of items that exceed the capacity.
    pub excess_count: ItemCount,
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
    pub missing_count: ItemCount,
}

/// Failed to remove many items from an inventory.
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveManyItemsError {
    /// The number of items that were missing from the inventory.
    pub missing_counts: Vec<ItemCount>,
}

/// Failed to completely transfer items from one inventory to another.
pub struct ItemTransferError {
    /// The number and type of items remaining in the input that could not be transferred.
    pub items_remaining: usize,
    /// Did this fail because the input inventory was full?
    pub full_sink: bool,
    /// Did this fail because the output inventory was empty?
    pub empty_source: bool,
}
