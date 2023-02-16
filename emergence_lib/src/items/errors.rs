//! Errors related to items and inventories.

use super::ItemCount;

/// Failed to add items to an inventory.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AddOneItemError {
    /// The number of items that exceed the capacity.
    pub excess_count: usize,
}

/// Failed to add items to an inventory.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AddManyItemsError {
    /// The number of items that exceeded the capacity.
    pub excess_counts: Vec<ItemCount>,
}

/// Failed to remove items from an item slot.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemoveOneItemError {
    /// The number of items that were missing from the inventory.
    pub(crate) missing_count: usize,
}

/// Failed to remove many items from an inventory.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemoveManyItemsError {
    /// The number of items that were missing from the inventory.
    pub(crate) missing_counts: Vec<ItemCount>,
}

/// Failed to completely transfer items from one inventory to another.
pub(crate) struct ItemTransferError {
    /// The number and type of items remaining in the input that could not be transferred.
    pub(crate) items_remaining: usize,
    /// Did this fail because the input inventory was full?
    pub(crate) full_sink: bool,
    /// Did this fail because the output inventory was empty?
    pub(crate) empty_source: bool,
}
