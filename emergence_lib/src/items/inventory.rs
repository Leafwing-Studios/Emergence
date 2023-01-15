//! Storage of multiple items with a capacity.

use std::fmt::Display;

use super::{
    count::ItemCount,
    errors::{AddManyItemsError, AddOneItemError, RemoveManyItemsError, RemoveOneItemError},
    slot::ItemSlot,
    ItemId,
};

/// An inventory to store multiple types of items.
#[derive(Debug, Default, Clone)]
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
                if slot.is_for_item(item_id) {
                    Some(slot.count())
                } else {
                    None
                }
            })
            .sum()
    }

    /// Determine if the inventory holds enough of the given item.
    pub fn has_count_of_item(&self, item_count: &ItemCount) -> bool {
        self.item_count(item_count.item_id()) >= item_count.count()
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
        self.max_slot_count - self.slots.len()
    }

    /// The number of items of the given type that can still fit in the inventory.
    pub fn remaining_space_for_item(&self, item_id: &ItemId) -> usize {
        // We can fill up the remaining space in the slots for this item...
        self.slots
            .iter()
            .filter_map(|slot| {
                if slot.is_for_item(item_id) {
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
    pub fn add_until_full_one_item(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), AddOneItemError> {
        let mut items_to_add = item_count.count();

        // Fill up the slots of this item
        for slot in self
            .slots
            .iter_mut()
            .filter(|slot| slot.is_for_item(item_count.item_id()))
        {
            match slot.add_until_full(items_to_add) {
                Ok(_) => {
                    items_to_add = 0;
                    break;
                }
                Err(AddOneItemError { excess_count }) => items_to_add = excess_count.count(),
            }
        }

        // Fill up the remaining free slots
        while items_to_add > 0 && self.slots.len() < self.max_slot_count {
            let mut new_slot = ItemSlot::new(item_count.item_id().clone(), self.max_items_per_slot);

            match new_slot.add_until_full(items_to_add) {
                Ok(_) => {
                    items_to_add = 0;
                }
                Err(AddOneItemError { excess_count }) => items_to_add = excess_count.count(),
            }

            self.slots.push(new_slot);
        }

        // Make sure that the invariants still hold
        debug_assert!(self.slots.len() <= self.max_slot_count);

        if items_to_add > 0 {
            Err(AddOneItemError {
                excess_count: ItemCount::new(item_count.item_id().clone(), items_to_add),
            })
        } else {
            Ok(())
        }
    }

    /// Try add the given count of items to the inventory, together.
    ///
    /// - If the items can fit in the slot, they are all added and `Ok` is returned.
    /// - If at least one of the items does not fit, _no_ items are added and `Err` is returned.
    pub fn add_all_or_nothing_one_item(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), AddOneItemError> {
        let remaining_space = self.remaining_space_for_item(item_count.item_id());

        if remaining_space < item_count.count() {
            Err(AddOneItemError {
                excess_count: ItemCount::new(
                    item_count.item_id().clone(),
                    item_count.count() - remaining_space,
                ),
            })
        } else {
            // If this unwrap panics the remaining space calculation must be wrong
            self.add_until_full_one_item(item_count).unwrap();

            Ok(())
        }
    }

    /// Try to add all the given items at once.
    ///
    /// - If at least one item doesn't fit in the inventory, _no_ items are added and `Err` is returned.
    /// - Otherwise, the given items are all added to the inventory.
    pub fn add_all_or_nothing_many_items(
        &mut self,
        item_counts: &[ItemCount],
    ) -> Result<(), AddManyItemsError> {
        let excess_counts: Vec<ItemCount> = item_counts
            .iter()
            .filter_map(|item_count| {
                let excess = item_count
                    .count()
                    .saturating_sub(self.remaining_space_for_item(item_count.item_id()));

                if excess > 0 {
                    Some(ItemCount::new(item_count.item_id().clone(), excess))
                } else {
                    None
                }
            })
            .collect();

        if excess_counts.is_empty() {
            item_counts
                .iter()
                .for_each(|item_count| self.add_all_or_nothing_one_item(item_count).unwrap());
            Ok(())
        } else {
            Err(AddManyItemsError { excess_counts })
        }
    }

    /// Try to remove as many items from the inventory as possible, up to the given count.
    ///
    /// - If the slot has enough items, they are all removed and `Ok` is returned.
    /// - Otherwise, all items that are included are removed and `Err` is returned.
    pub fn remove_until_empty_one_item(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), RemoveOneItemError> {
        let mut items_to_remove = item_count.count();
        let mut has_to_clear_slots = false;

        for slot in self
            .slots
            .iter_mut()
            .filter(|slot| slot.is_for_item(item_count.item_id()))
            .rev()
        {
            match slot.remove_until_empty(items_to_remove) {
                Ok(_) => {
                    items_to_remove = 0;
                    break;
                }
                Err(RemoveOneItemError {
                    missing_count: excess_count,
                }) => {
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
            Err(RemoveOneItemError {
                missing_count: items_to_remove,
            })
        } else {
            Ok(())
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// - If there are enough items in the slot, they are all removed and `Ok` is returned.
    /// - If there are not enough items, _no_ item is removed and `Err` is returned.
    pub fn remove_all_or_nothing_one_item(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), RemoveOneItemError> {
        let cur_count = self.item_count(item_count.item_id());

        if cur_count < item_count.count() {
            Err(RemoveOneItemError {
                missing_count: item_count.count() - cur_count,
            })
        } else {
            // If this unwrap panics the removal or the item counting must be wrong
            self.remove_until_empty_one_item(item_count).unwrap();
            Ok(())
        }
    }

    /// Try to remove all the given items from the inventory.
    ///
    /// - If there are not enough items from any item type, `Err` is returned and _no_ items are removed.
    /// - If there are enough items, they are all removed and `Ok` is returned.
    pub fn remove_all_or_nothing_many_items(
        &mut self,
        item_counts: &[ItemCount],
    ) -> Result<(), RemoveManyItemsError> {
        let missing_counts: Vec<ItemCount> = item_counts
            .iter()
            .filter_map(|item_count| {
                let missing = self
                    .item_count(item_count.item_id())
                    .saturating_sub(item_count.count());

                if missing > 0 {
                    Some(ItemCount::new(item_count.item_id().clone(), missing))
                } else {
                    None
                }
            })
            .collect();

        if missing_counts.is_empty() {
            item_counts
                .iter()
                .for_each(|item_count| self.add_all_or_nothing_one_item(item_count).unwrap());
            Ok(())
        } else {
            Err(RemoveManyItemsError { missing_counts })
        }
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slot_strings: Vec<String> = self
            // Filled slots
            .slots
            .iter()
            .map(|slot| format!("{slot}"))
            // Empty slots
            .chain((0..self.free_slot_count()).map(|_| "_".to_string()))
            .collect();

        write!(f, "[{}]", slot_strings.join(", "))
    }
}
