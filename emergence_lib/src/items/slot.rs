//! A container for a single item type, with a capacity.

use std::fmt::Display;

use super::{
    count::ItemCount,
    errors::{AddOneItemError, RemoveOneItemError},
    ItemId,
};

/// Multiple items of the same type.
#[derive(Debug, Clone)]
pub struct ItemSlot {
    /// The unique identifier of the item that occupies the slot.
    item_id: ItemId,

    /// The maximum number of items that fit in the slot.
    max_item_count: usize,

    /// The number of items in the slot.
    ///
    /// This is guaranteed to be smaller than or equal to the `max_item_count`.
    count: usize,
}

impl ItemSlot {
    /// Create an empty slot for the given item.
    pub fn new(item_id: ItemId, max_item_count: usize) -> Self {
        Self {
            item_id,
            max_item_count,
            count: 0,
        }
    }

    /// The unique identifier of the item in the slot.
    pub fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    /// The number of items in this slot.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The maximum number of items that can fit in this slot.
    pub fn max_item_count(&self) -> usize {
        self.max_item_count
    }

    /// The number of items that can still fit in the item slot.
    pub fn remaining_space(&self) -> usize {
        self.max_item_count - self.count
    }

    /// Returns `true` if there are no items stored in this slot.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns `true` if the maximum item count of this slot has been reached.
    pub fn is_full(&self) -> bool {
        self.count == self.max_item_count
    }

    /// Determine if this slot can hold items of the given type.
    pub fn is_for_item(&self, item_id: &ItemId) -> bool {
        self.item_id == *item_id
    }

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// - If all items can fit in the slot, they are all added and `Ok` is returned.
    /// - Otherwise, all items that can fit are added and `Err` is returned.
    pub fn add_until_full(&mut self, count: usize) -> Result<(), AddOneItemError> {
        let new_count = self.count + count;

        if new_count > self.max_item_count {
            self.count = self.max_item_count;

            Err(AddOneItemError {
                excess_count: ItemCount::new(self.item_id.clone(), new_count - self.max_item_count),
            })
        } else {
            self.count = new_count;
            Ok(())
        }
    }

    /// Try to add the given count of items to the inventory, together.
    ///
    /// - If the items can fit in the slot, they are all added and `Ok` is returned.
    /// - If at least one of the items does not fit, _no_ items are added and `Err` is returned.
    pub fn add_all_or_nothing(&mut self, count: usize) -> Result<(), AddOneItemError> {
        if self.remaining_space() < count {
            Err(AddOneItemError {
                excess_count: ItemCount::new(
                    self.item_id.clone(),
                    count - (self.max_item_count - self.count),
                ),
            })
        } else {
            self.count += count;
            Ok(())
        }
    }

    /// Try to remove as many items from the slot as possible, up to the given count.
    ///
    /// - If the slot has enough items, they are all removed and `Ok` is returned.
    /// - Otherwise, all items that are included are removed and `Err` is returned.
    pub fn remove_until_empty(&mut self, count: usize) -> Result<(), RemoveOneItemError> {
        if count > self.count {
            let excess_count = count - self.count;
            self.count = 0;

            Err(RemoveOneItemError {
                missing_count: excess_count,
            })
        } else {
            self.count -= count;
            Ok(())
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// - If there are enough items in the slot, they are all removed and `Ok` is returned.
    /// - If there are not enough items, _no_ item is removed and `Err` is returned.
    pub fn remove_all_or_nothing(&mut self, count: usize) -> Result<(), RemoveOneItemError> {
        if count > self.count {
            Err(RemoveOneItemError {
                missing_count: count - self.count,
            })
        } else {
            self.count -= count;
            Ok(())
        }
    }
}

impl Display for ItemSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}/{})",
            self.item_id, self.count, self.max_item_count
        )
    }
}
