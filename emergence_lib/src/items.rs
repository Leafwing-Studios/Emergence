//! Everything related to items and crafting.

/// The unique identifier of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemId(&'static str);

/// Failed to add items to an item slot.
#[derive(Debug)]
pub struct AddItemsError {
    /// The number of items that exceed the capacity.
    pub excess_count: usize,
}

/// Failed to remove items from an item slot.
#[derive(Debug)]
pub struct RemoveItemsError {
    /// The number of items that were missing from the slot.
    pub excess_count: usize,
}

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

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// - If all items can fit in the slot, they are all added and `Ok` is returned.
    /// - Otherwise, all items that can fit are added and `Err` is returned.
    pub fn add_until_full(&mut self, count: usize) -> Result<(), AddItemsError> {
        let new_count = self.count + count;

        if new_count > self.max_item_count {
            self.count = self.max_item_count;

            Err(AddItemsError {
                excess_count: new_count - self.max_item_count,
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
    pub fn add_all_or_nothing(&mut self, count: usize) -> Result<(), AddItemsError> {
        if self.remaining_space() < count {
            Err(AddItemsError {
                excess_count: count - (self.max_item_count - self.count),
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
    pub fn remove_until_empty(&mut self, count: usize) -> Result<(), RemoveItemsError> {
        if count > self.count {
            let excess_count = count - self.count;
            self.count = 0;

            Err(RemoveItemsError { excess_count })
        } else {
            self.count -= count;
            Ok(())
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// - If there are enough items in the slot, they are all removed and `Ok` is returned.
    /// - If there are not enough items, _no_ item is removed and `Err` is returned.
    pub fn remove_all_or_nothing(&mut self, count: usize) -> Result<(), RemoveItemsError> {
        if count > self.count {
            Err(RemoveItemsError {
                excess_count: count - self.count,
            })
        } else {
            self.count -= count;
            Ok(())
        }
    }
}

/// An inventory to store multiple types of items.
#[derive(Debug, Clone)]
pub struct Inventory {
    /// The item slots that are currently active.
    ///
    /// `slots.len() <= max_slot_count` is guaranteed.
    slots: Vec<ItemSlot>,

    /// The maximum number of item slots this inventory can hold.
    max_slot_count: usize,

    /// The number of items that can fit in each slot.
    ///
    /// In the future we will probably handle this differently.
    /// For example, different slot sizes per item type.
    max_items_per_slot: usize,
}

impl Inventory {
    /// Create an empty inventory with the given amount of slots.
    pub fn new(max_slot_count: usize, max_items_per_slot: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slot_count,
            max_items_per_slot,
        }
    }

    /// Determine how many items of the given type are in the inventory.
    pub fn item_count(&self, item_id: &ItemId) -> usize {
        self.slots
            .iter()
            .filter_map(|slot| {
                if slot.item_id() == item_id {
                    Some(slot.count())
                } else {
                    None
                }
            })
            .sum()
    }

    /// Determine if the inventory holds enough of the given item.
    pub fn has_count_of_item(&self, item_id: &ItemId, count: usize) -> bool {
        self.item_count(item_id) >= count
    }

    /// Returns `true` if there are no items in the inventory.
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_empty())
    }

    /// Returns `true` if all slots are filled to their capacity.
    pub fn is_full(&self) -> bool {
        self.slots.len() == self.max_slot_count && self.slots.iter().all(|slot| slot.is_full())
    }

    /// The number of slots that don't have an item in them.
    pub fn free_slot_count(&self) -> usize {
        self.max_items_per_slot - self.slots.len()
    }

    /// The number of items of the given type that can still fit in the inventory.
    pub fn remaining_space_for_item(&self, item_id: &ItemId) -> usize {
        // We can fill up the remaining space in the slots for this item...
        self.slots
            .iter()
            .filter_map(|slot| {
                if slot.item_id() == item_id {
                    Some(slot.remaining_space())
                } else {
                    None
                }
            })
            .sum::<usize>()
            // ...and use up the remaining free slots
            + self.free_slot_count() * self.max_items_per_slot
    }

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// - If all items can fit in the slot, they are all added and `Ok` is returned.
    /// - Otherwise, all items that can fit are added and `Err` is returned.
    pub fn add_until_full(&mut self, item_id: &ItemId, count: usize) -> Result<(), AddItemsError> {
        let mut items_to_add = count;

        // Fill up the slots of this item
        for slot in self
            .slots
            .iter_mut()
            .filter(|slot| slot.item_id() == item_id)
        {
            match slot.add_until_full(items_to_add) {
                Ok(_) => {
                    items_to_add = 0;
                    break;
                }
                Err(AddItemsError { excess_count }) => items_to_add = excess_count,
            }
        }

        // Fill up the remaining free slots
        while items_to_add > 0 && self.slots.len() < self.max_slot_count {
            let mut new_slot = ItemSlot::new(item_id.clone(), self.max_items_per_slot);

            match new_slot.add_until_full(items_to_add) {
                Ok(_) => {
                    items_to_add = 0;
                    break;
                }
                Err(AddItemsError { excess_count }) => items_to_add = excess_count,
            }

            self.slots.push(new_slot);
        }

        // Make sure that the invariants still hold
        debug_assert!(self.slots.len() <= self.max_slot_count);

        if items_to_add > 0 {
            Err(AddItemsError {
                excess_count: items_to_add,
            })
        } else {
            Ok(())
        }
    }

    /// Try add the given count of items to the inventory, together.
    ///
    /// - If the items can fit in the slot, they are all added and `Ok` is returned.
    /// - If at least one of the items does not fit, _no_ items are added and `Err` is returned.
    pub fn add_all_or_nothing(
        &mut self,
        item_id: &ItemId,
        count: usize,
    ) -> Result<(), AddItemsError> {
        let remaining_space = self.remaining_space_for_item(item_id);

        if remaining_space < count {
            Err(AddItemsError {
                excess_count: count - remaining_space,
            })
        } else {
            // If this unwrap panics the remaining space calculation must be wrong
            self.add_until_full(item_id, count).unwrap();

            Ok(())
        }
    }

    /// Try to remove as many items from the inventory as possible, up to the given count.
    ///
    /// - If the slot has enough items, they are all removed and `Ok` is returned.
    /// - Otherwise, all items that are included are removed and `Err` is returned.
    pub fn remove_until_empty(
        &mut self,
        item_id: &ItemId,
        count: usize,
    ) -> Result<(), RemoveItemsError> {
        let mut items_to_remove = count;
        let mut has_to_clear_slots = false;

        for slot in self
            .slots
            .iter_mut()
            .filter(|slot| slot.item_id() == item_id)
            .rev()
        {
            match slot.remove_until_empty(items_to_remove) {
                Ok(_) => {
                    items_to_remove = 0;
                    break;
                }
                Err(RemoveItemsError { excess_count }) => {
                    items_to_remove = excess_count;
                    has_to_clear_slots = true;
                }
            }
        }

        // If a slot now has 0 items remove it
        // This makes space for other item types
        if has_to_clear_slots {
            self.slots = self
                .slots
                .iter()
                .cloned()
                .filter(|slot| !slot.is_empty())
                .collect();
        }

        if items_to_remove > 0 {
            Err(RemoveItemsError {
                excess_count: items_to_remove,
            })
        } else {
            Ok(())
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// - If there are enough items in the slot, they are all removed and `Ok` is returned.
    /// - If there are not enough items, _no_ item is removed and `Err` is returned.
    pub fn remove_all_or_nothing(
        &mut self,
        item_id: &ItemId,
        count: usize,
    ) -> Result<(), RemoveItemsError> {
        let item_count = self.item_count(item_id);

        if item_count < count {
            Err(RemoveItemsError {
                excess_count: count - item_count,
            })
        } else {
            // If this unwrap panics the removal or the item counting must be wrong
            self.remove_until_empty(item_id, count).unwrap();
            Ok(())
        }
    }
}
