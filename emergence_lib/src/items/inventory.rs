//! Storage of multiple items with a capacity.

use std::fmt::Display;

use super::{
    errors::{
        AddManyItemsError, AddOneItemError, ItemTransferError, RemoveManyItemsError,
        RemoveOneItemError,
    },
    slot::ItemSlot,
    ItemCount, ItemId, ItemManifest,
};

/// An inventory to store multiple types of items.
#[derive(Debug, Default, Clone)]
pub(crate) struct Inventory {
    /// The item slots that are currently active.
    ///
    /// `slots.len() <= max_slot_count` is guaranteed.
    slots: Vec<ItemSlot>,

    /// The maximum number of item slots this inventory can hold.
    max_slot_count: usize,
}

/// The fullness of an inventory
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub(crate) enum InventoryState {
    /// Fully empty.
    Empty,
    /// Neither empty nor full.
    #[default]
    Partial,
    /// Completely full.
    Full,
}

impl InventoryState {
    /// Combines `self` with `other`, yielding a new [`InventoryState`].
    ///
    /// Empty + Empty = Empty
    /// Full + Full = Full
    /// Otherwise, Partial
    fn combine(&self, other: InventoryState) -> Self {
        use InventoryState::*;
        match (self, other) {
            (Empty, Empty) => Empty,
            (Full, Full) => Full,
            _ => Partial,
        }
    }
}

#[allow(dead_code)]
impl Inventory {
    /// Create an empty inventory with the given amount of slots.
    pub(crate) fn new(max_slot_count: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slot_count,
        }
    }

    // FIXME: this doesn't properly respect max stack size
    /// Creates an inventory from the provided [`ItemCount`].
    pub(crate) fn new_from_item(item_count: ItemCount) -> Self {
        Self {
            slots: vec![ItemSlot::new(item_count.item_id, item_count.count)],
            max_slot_count: 1,
        }
    }

    /// Returns an iterator over the items in the inventory and their count.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &ItemSlot> {
        self.slots.iter()
    }

    /// Returns a mutable iterator over the contained item slots.
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut ItemSlot> {
        self.slots.iter_mut()
    }

    /// How full is this inventory?
    pub(crate) fn state(&self) -> InventoryState {
        let mut inventory_state: Option<InventoryState> = None;

        for item_slot in self.iter() {
            let slot_state = item_slot.state();
            inventory_state = match inventory_state {
                Some(previous_state) => Some(previous_state.combine(slot_state)),
                None => Some(slot_state),
            };

            // Partially filled inventories are always partially filled.
            if matches!(inventory_state.unwrap(), InventoryState::Partial) {
                return InventoryState::Partial;
            }
        }

        inventory_state.unwrap_or_default()
    }

    /// Determine how many items of the given type are in the inventory.
    pub(crate) fn item_count(&self, item_id: ItemId) -> usize {
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
    pub(crate) fn has_count_of_item(&self, item_count: &ItemCount) -> bool {
        self.item_count(item_count.item_id()) >= item_count.count()
    }

    /// Returns `true` if there are no items in the inventory.
    pub(crate) fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_empty())
    }

    /// Returns `true` if all slots are filled to their capacity.
    pub(crate) fn is_full(&self) -> bool {
        self.slots.len() == self.max_slot_count && self.slots.iter().all(|slot| slot.is_full())
    }

    /// The number of slots that don't have an item in them.
    pub(crate) fn free_slot_count(&self) -> usize {
        self.max_slot_count - self.slots.len()
    }

    /// The remaining space for the item in the slots that it already occupies.
    pub(crate) fn remaining_reserved_space_for_item(&self, item_id: ItemId) -> usize {
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
    }

    /// The number of items of the given type that can still fit in the inventory.
    pub(crate) fn remaining_space_for_item(
        &self,
        item_id: ItemId,
        item_manifest: &ItemManifest,
    ) -> usize {
        // We can fill up the remaining space in the slots for this item...
        self.remaining_reserved_space_for_item(item_id)
            // ...and use up the remaining free slots
            + self.free_slot_count() * item_manifest.get(item_id).stack_size()
    }

    /// Clears any inventory stacks with 0 items in them.
    ///
    /// This is the standard behavior for units and storages, but not for crafting.
    /// In those cases, the slots should persist with 0 items.
    pub(crate) fn clear_empty_slots(&mut self) {
        let mut slots_to_clear: Vec<usize> = Vec::with_capacity(self.max_slot_count);

        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_empty() {
                slots_to_clear.push(i);
            }
        }

        for i in slots_to_clear {
            self.slots.remove(i);
        }
    }

    /// Adds an empty slot that is reserved for the provided `item_id`.
    ///
    /// This operation is infallible: if there are not enough slots available, the inventory size will be expanded.
    pub(crate) fn add_empty_slot(&mut self, item_id: ItemId, item_manifest: &ItemManifest) {
        let n_existing_slots = self.slots.len();
        let slot_to_use = n_existing_slots + 1;
        let stack_size = item_manifest.get(item_id).stack_size();
        let empty_stack = ItemSlot::new(item_id, stack_size);

        if slot_to_use >= self.max_slot_count {
            self.max_slot_count = slot_to_use;
            self.slots.push(empty_stack);
        } else {
            self.slots[slot_to_use] = empty_stack
        }
    }

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// Items can spill over, filling multiple inventory slots at once if the amount to add is greater than the stack size.
    ///
    /// - If all items can fit in the inventory, they are all added and `Ok` is returned.
    /// - Otherwise, all items that can fit are added and `Err` is returned.
    ///
    /// # Warning
    ///
    /// Adding 0 of an item will not create an empty slot. Instead, use [`Inventory::add_empty_slot`].
    pub(crate) fn try_add_item(
        &mut self,
        item_count: &ItemCount,
        item_manifest: &ItemManifest,
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
            let mut new_slot = ItemSlot::new(
                item_count.item_id(),
                item_manifest.get(item_count.item_id()).stack_size,
            );

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
                excess_count: ItemCount::new(item_count.item_id(), items_to_add),
            })
        } else {
            Ok(())
        }
    }

    /// Try add the given count of items to the inventory, together.
    ///
    /// - If the items can fit in the slot, they are all added and `Ok` is returned.
    /// - If at least one of the items does not fit, _no_ items are added and `Err` is returned.
    pub fn add_item_all_or_nothing(
        &mut self,
        item_count: &ItemCount,
        item_manifest: &ItemManifest,
    ) -> Result<(), AddOneItemError> {
        let remaining_space = self.remaining_space_for_item(item_count.item_id(), item_manifest);

        if remaining_space < item_count.count() {
            Err(AddOneItemError {
                excess_count: ItemCount::new(
                    item_count.item_id(),
                    item_count.count() - remaining_space,
                ),
            })
        } else {
            // If this unwrap panics the remaining space calculation must be wrong
            self.try_add_item(item_count, item_manifest).unwrap();

            Ok(())
        }
    }

    /// Try to add all the given items at once.
    ///
    /// - If at least one item doesn't fit in the inventory, _no_ items are added and `Err` is returned.
    /// - Otherwise, the given items are all added to the inventory.
    ///
    /// The item counts must not contain any duplicates.
    pub fn add_items_all_or_nothing(
        &mut self,
        item_counts: &[ItemCount],
        item_manifest: &ItemManifest,
    ) -> Result<(), AddManyItemsError> {
        let mut free_slot_count = self.free_slot_count();

        let excess_counts: Vec<ItemCount> = item_counts
            .iter()
            .filter_map(|item_count| {
                let stack_size = item_manifest.get(item_count.item_id()).stack_size;

                let remaining_reserved_space =
                    self.remaining_reserved_space_for_item(item_count.item_id());
                let remaining_free_space = free_slot_count * stack_size;

                let excess = item_count
                    .count()
                    .saturating_sub(remaining_reserved_space + remaining_free_space);

                if item_count.count() > remaining_reserved_space {
                    // Update the count of the remaining free slots
                    free_slot_count = free_slot_count.saturating_sub(
                        (item_count.count().saturating_sub(remaining_reserved_space) as f32
                            / stack_size as f32)
                            .ceil() as usize,
                    );
                }

                if excess > 0 {
                    Some(ItemCount::new(item_count.item_id(), excess))
                } else {
                    None
                }
            })
            .collect();

        if excess_counts.is_empty() {
            item_counts.iter().for_each(|item_count| {
                self.add_item_all_or_nothing(item_count, item_manifest)
                    .unwrap()
            });
            Ok(())
        } else {
            Err(AddManyItemsError { excess_counts })
        }
    }

    /// Adds as many items as possible, overflowing the rest if they cannot be added
    pub fn try_add_items(
        &mut self,
        item_counts: &[ItemCount],
        item_manifest: &ItemManifest,
    ) -> Result<(), AddManyItemsError> {
        let mut overflow: Vec<ItemCount> = Vec::new();

        for item_count in item_counts {
            if let Err(AddOneItemError { excess_count }) =
                self.try_add_item(item_count, item_manifest)
            {
                overflow.push(excess_count);
            }
        }

        if overflow.is_empty() {
            Ok(())
        } else {
            Err(AddManyItemsError {
                excess_counts: overflow,
            })
        }
    }

    /// Try to remove as many items from the inventory as possible, up to the given count.
    ///
    /// - If the slot has enough items, they are all removed and `Ok` is returned.
    /// - Otherwise, all items that are included are removed and `Err` is returned.
    pub fn try_remove_item(&mut self, item_count: &ItemCount) -> Result<(), RemoveOneItemError> {
        let mut items_to_remove = item_count.count();

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
                Err(RemoveOneItemError { missing_count }) => {
                    items_to_remove = missing_count.count();
                }
            }
        }

        if items_to_remove > 0 {
            Err(RemoveOneItemError {
                missing_count: ItemCount::new(item_count.item_id(), items_to_remove),
            })
        } else {
            Ok(())
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// - If there are enough items in the slot, they are all removed and `Ok` is returned.
    /// - If there are not enough items, _no_ item is removed and `Err` is returned.
    pub fn remove_item_all_or_nothing(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), RemoveOneItemError> {
        let cur_count = self.item_count(item_count.item_id());

        if cur_count < item_count.count() {
            Err(RemoveOneItemError {
                missing_count: ItemCount::new(item_count.item_id(), item_count.count() - cur_count),
            })
        } else {
            // If this unwrap panics the removal or the item counting must be wrong
            self.try_remove_item(item_count).unwrap();
            Ok(())
        }
    }

    /// Try to remove all the given items from the inventory.
    ///
    /// - If there are not enough items from any item type, `Err` is returned and _no_ items are removed.
    /// - If there are enough items, they are all removed and `Ok` is returned.
    pub fn remove_items_all_or_nothing(
        &mut self,
        item_counts: &[ItemCount],
    ) -> Result<(), RemoveManyItemsError> {
        let missing_counts: Vec<ItemCount> = item_counts
            .iter()
            .filter_map(|item_count| {
                let missing = item_count
                    .count()
                    .saturating_sub(self.item_count(item_count.item_id()));

                if missing > 0 {
                    Some(ItemCount::new(item_count.item_id(), missing))
                } else {
                    None
                }
            })
            .collect();

        if missing_counts.is_empty() {
            item_counts
                .iter()
                .for_each(|item_count| self.remove_item_all_or_nothing(item_count).unwrap());
            Ok(())
        } else {
            Err(RemoveManyItemsError { missing_counts })
        }
    }

    /// Transfers item of the type given by `item_count` from the inventory of `self` to `other`.
    ///
    /// As many items will be transferred as possible.
    /// If all items are transferred, [`Ok(())`] will be returned.
    /// Otherwise, an [`ItemTransferError`] will be returned that contains the number of items that could not be transferred and why.
    pub fn transfer_item(
        &mut self,
        item_count: &ItemCount,
        other: &mut Inventory,
        item_manifest: &ItemManifest,
    ) -> Result<(), ItemTransferError> {
        let item_id = item_count.item_id();

        let requested = item_count.count();
        let available = self.item_count(item_id);
        let free = other.remaining_space_for_item(item_id, item_manifest);

        let proposed = requested.min(available);
        let actual = proposed.min(free);

        // Skip the expensive work if there's nothing to move
        if actual > 0 {
            let actual_count = ItemCount::new(item_id, actual);

            // Unwraps are being used as assertions here: if this is panicking, this method is broken
            self.remove_item_all_or_nothing(&actual_count).unwrap();
            other
                .add_item_all_or_nothing(&actual_count, item_manifest)
                .unwrap();
        }

        if actual == requested {
            Ok(())
        } else {
            Err(ItemTransferError {
                items_remaining: ItemCount::new(
                    item_count.item_id(),
                    available.saturating_sub(actual),
                ),
                full_destination: proposed > free,
                empty_source: requested > available,
            })
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

#[cfg(test)]
mod tests {
    use bevy::utils::HashMap;

    use super::*;
    use crate::items::{ItemData, ItemId};

    /// Create a simple item manifest for testing purposes.
    fn item_manifest() -> ItemManifest {
        let mut item_manifest = HashMap::new();
        item_manifest.insert(ItemId::acacia_leaf(), ItemData::acacia_leaf());
        item_manifest.insert(ItemId::test(), ItemData { stack_size: 10 });

        ItemManifest::new(item_manifest)
    }

    fn full_inventory() -> Inventory {
        Inventory {
            max_slot_count: 1,
            slots: vec![ItemSlot::new_with_count(ItemId::test(), 10, 10)],
        }
    }

    fn partial_inventory() -> Inventory {
        Inventory {
            max_slot_count: 1,
            slots: vec![ItemSlot::new_with_count(ItemId::test(), 10, 7)],
        }
    }

    fn empty_inventory() -> Inventory {
        Inventory {
            max_slot_count: 1,
            slots: vec![],
        }
    }

    fn transfer_count() -> ItemCount {
        ItemCount {
            item_id: ItemId::test(),
            count: 10,
        }
    }

    mod display {
        use super::super::*;

        #[test]
        fn should_display_with_no_slots() {
            let inventory = Inventory::new(0);

            assert_eq!(format!("{inventory}"), "[]".to_string());
        }

        #[test]
        fn should_display_with_empty_slot() {
            let inventory = Inventory::new(1);

            assert_eq!(format!("{inventory}"), "[_]".to_string());
        }

        #[test]
        fn should_display_with_filled_slot() {
            let inventory = Inventory {
                max_slot_count: 1,
                slots: vec![ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5)],
            };

            assert_eq!(format!("{inventory}"), "[acacia_leaf (5/10)]".to_string());
        }
    }

    #[test]
    fn should_count_item() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 15);
    }

    #[test]
    fn should_determine_that_item_count_is_available() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert!(inventory.has_count_of_item(&ItemCount::new(ItemId::acacia_leaf(), 15)));
    }

    #[test]
    fn should_determine_that_item_count_is_not_available() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert!(!inventory.has_count_of_item(&ItemCount::new(ItemId::acacia_leaf(), 16)));
    }

    #[test]
    fn should_determine_that_inventory_is_empty() {
        let inventory = Inventory::new(4);

        assert!(inventory.is_empty());
    }

    #[test]
    fn should_determine_that_inventory_is_not_empty() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert!(!inventory.is_empty());
    }

    #[test]
    fn should_determine_that_inventory_is_full() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
            ],
        };

        assert!(inventory.is_full());
    }

    #[test]
    fn should_determine_that_inventory_is_not_full() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert!(!inventory.is_full());
    }

    #[test]
    fn should_calculate_number_of_free_slots() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert_eq!(inventory.free_slot_count(), 1);
    }

    #[test]
    fn should_calculate_remaining_space_for_item() {
        let inventory = Inventory {
            max_slot_count: 4,
            slots: vec![
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                ItemSlot::new_with_count(ItemId::test(), 10, 3),
            ],
        };

        assert_eq!(
            inventory.remaining_space_for_item(ItemId::acacia_leaf(), &item_manifest()),
            15
        );
    }

    mod add {
        mod until_full_one_item {
            use super::super::item_manifest;

            use super::super::super::*;

            #[test]
            fn should_be_ok_when_all_fit() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory
                        .try_add_item(&ItemCount::new(ItemId::acacia_leaf(), 15), &item_manifest()),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 30);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }

            #[test]
            fn should_fill_up_when_not_all_fit() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory
                        .try_add_item(&ItemCount::new(ItemId::acacia_leaf(), 20), &item_manifest()),
                    Err(AddOneItemError {
                        excess_count: ItemCount {
                            item_id: ItemId::acacia_leaf(),
                            count: 5
                        }
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 30);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }

        mod all_or_nothing_one_item {
            use super::super::super::*;
            use super::super::item_manifest;

            #[test]
            fn should_be_ok_when_all_fit() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.add_item_all_or_nothing(
                        &ItemCount::new(ItemId::acacia_leaf(), 15),
                        &item_manifest()
                    ),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 30);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }

            #[test]
            fn should_not_add_anything_if_not_enough_space() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.add_item_all_or_nothing(
                        &ItemCount::new(ItemId::acacia_leaf(), 16),
                        &item_manifest()
                    ),
                    Err(AddOneItemError {
                        excess_count: ItemCount::new(ItemId::acacia_leaf(), 1)
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 15);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }

        mod all_or_nothing_many_items {
            use super::super::super::*;
            use super::super::item_manifest;

            #[test]
            fn should_be_ok_when_all_fit() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.add_items_all_or_nothing(
                        &[
                            ItemCount::new(ItemId::acacia_leaf(), 15),
                            ItemCount::new(ItemId::test(), 7)
                        ],
                        &item_manifest()
                    ),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 30);
                assert_eq!(inventory.item_count(ItemId::test()), 10);
            }

            #[test]
            fn should_not_add_anything_if_not_enough_space() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.add_items_all_or_nothing(
                        &[
                            ItemCount::new(ItemId::acacia_leaf(), 15),
                            ItemCount::new(ItemId::test(), 8)
                        ],
                        &item_manifest()
                    ),
                    Err(AddManyItemsError {
                        excess_counts: vec![ItemCount::new(ItemId::test(), 1)]
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 15);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }
    }

    mod remove {
        mod until_empty_one_item {
            use super::super::super::*;

            #[test]
            fn should_be_ok_when_all_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.try_remove_item(&ItemCount::new(ItemId::acacia_leaf(), 15)),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 0);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }

            #[test]
            fn should_empty_when_not_all_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.try_remove_item(&ItemCount::new(ItemId::acacia_leaf(), 20)),
                    Err(RemoveOneItemError {
                        missing_count: ItemCount::new(ItemId::acacia_leaf(), 5)
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 0);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }

        mod all_or_nothing_one_item {
            use super::super::super::*;

            #[test]
            fn should_be_ok_when_all_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory
                        .remove_item_all_or_nothing(&ItemCount::new(ItemId::acacia_leaf(), 15)),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 0);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }

            #[test]
            fn should_not_remove_anything_if_not_enough_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory
                        .remove_item_all_or_nothing(&ItemCount::new(ItemId::acacia_leaf(), 16)),
                    Err(RemoveOneItemError {
                        missing_count: ItemCount::new(ItemId::acacia_leaf(), 1)
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 15);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }

        mod all_or_nothing_many_items {
            use super::super::super::*;

            #[test]
            fn should_be_ok_when_all_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.remove_items_all_or_nothing(&[
                        ItemCount::new(ItemId::acacia_leaf(), 15),
                        ItemCount::new(ItemId::test(), 3)
                    ]),
                    Ok(())
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 0);
                assert_eq!(inventory.item_count(ItemId::test()), 0);
            }

            #[test]
            fn should_not_remove_anything_if_not_enough_exist() {
                let mut inventory = Inventory {
                    max_slot_count: 4,
                    slots: vec![
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 10),
                        ItemSlot::new_with_count(ItemId::acacia_leaf(), 10, 5),
                        ItemSlot::new_with_count(ItemId::test(), 10, 3),
                    ],
                };

                assert_eq!(
                    inventory.remove_items_all_or_nothing(&[
                        ItemCount::new(ItemId::acacia_leaf(), 15),
                        ItemCount::new(ItemId::test(), 4)
                    ]),
                    Err(RemoveManyItemsError {
                        missing_counts: vec![ItemCount::new(ItemId::test(), 1)]
                    })
                );
                assert_eq!(inventory.item_count(ItemId::acacia_leaf()), 15);
                assert_eq!(inventory.item_count(ItemId::test()), 3);
            }
        }
    }

    mod transfer_item {
        use super::*;

        #[test]
        fn item_should_transfer_when_source_full() {
            let mut source = full_inventory();
            let mut destination = empty_inventory();

            let result =
                source.transfer_item(&transfer_count(), &mut destination, &item_manifest());
            assert!(result.is_ok());
        }

        #[test]
        fn item_should_not_transfer_when_source_empty() {
            let mut source = empty_inventory();
            let mut destination = empty_inventory();

            let result =
                source.transfer_item(&transfer_count(), &mut destination, &item_manifest());
            assert_eq!(
                result,
                Err(ItemTransferError {
                    items_remaining: ItemCount::new(ItemId::test(), 0),
                    full_destination: false,
                    empty_source: true
                })
            );
        }

        #[test]
        fn item_should_not_transfer_when_destination_full() {
            let mut source = full_inventory();
            let mut destination = full_inventory();

            let result =
                source.transfer_item(&transfer_count(), &mut destination, &item_manifest());
            assert_eq!(
                result,
                Err(ItemTransferError {
                    items_remaining: transfer_count(),
                    full_destination: true,
                    empty_source: false
                })
            );
        }

        #[test]
        fn not_enough_items() {
            let mut source = partial_inventory();
            let mut destination = empty_inventory();

            let result =
                source.transfer_item(&transfer_count(), &mut destination, &item_manifest());
            assert_eq!(
                result,
                Err(ItemTransferError {
                    items_remaining: ItemCount::new(ItemId::test(), 0),
                    full_destination: false,
                    empty_source: true
                })
            );
        }

        #[test]
        fn not_enough_space() {
            let mut source = full_inventory();
            let mut destination = partial_inventory();

            let result =
                source.transfer_item(&transfer_count(), &mut destination, &item_manifest());
            assert_eq!(
                result,
                Err(ItemTransferError {
                    items_remaining: ItemCount {
                        item_id: ItemId::test(),
                        count: 7
                    },
                    full_destination: true,
                    empty_source: false
                })
            );
        }
    }
}
