//! Everything related to items and crafting.

/// The unique identifier of an item.
#[derive(Debug, PartialEq, Eq)]
pub struct ItemId(&'static str);

/// Multiple items of the same type.
#[derive(Debug)]
pub struct ItemSlot {
    /// The unique identifier of the item that occupies the slot.
    item_id: ItemId,

    /// The maximum number of items that fit in the slot.
    capacity: usize,

    /// The number of items in the slot.
    ///
    /// This is guaranteed to be smaller than or equal to the `capacity`.
    count: usize,
}

/// Failed to add items to an item slot.
#[derive(Debug)]
pub struct AddItemsError {
    /// The number of items that exceed the capacity.
    pub excess_count: usize,
}

impl ItemSlot {
    /// Create an empty slot for the given item.
    pub fn new(item_id: ItemId, capacity: usize) -> Self {
        Self {
            item_id,
            capacity,
            count: 0,
        }
    }

    /// Try add the given count of items to the inventory, together.
    ///
    /// If the items can fit in the slot, they are all added.
    /// If at least one of the items does not fit, _no_ items are added.
    pub fn try_add_all(&mut self, count: usize) -> Result<(), AddItemsError> {
        if self.count + count > self.capacity {
            Err(AddItemsError {
                excess_count: count - (self.capacity - self.count),
            })
        } else {
            self.count += count;
            Ok(())
        }
    }

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// If all items can't fit in the capacity, the capacity is filled and an error is returned.
    pub fn try_add_some(&mut self, count: usize) -> Result<(), AddItemsError> {
        let new_count = self.count + count;

        if new_count > self.capacity {
            self.count = self.capacity;

            Err(AddItemsError {
                excess_count: new_count - self.capacity,
            })
        } else {
            self.count = new_count;
            Ok(())
        }
    }
}
