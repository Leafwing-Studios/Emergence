//! A container for a single item type, with a capacity.

use std::fmt::Display;

use super::{
    errors::{AddOneItemError, RemoveOneItemError},
    ItemId,
};

/// Multiple items of the same type.
#[derive(Debug, Clone)]
pub(crate) struct ItemSlot {
    /// The unique identifier of the item that occupies the slot.
    item_id: ItemId,

    /// The maximum number of items that fit in the slot.
    max_item_count: usize,

    /// The number of items in the slot.
    ///
    /// This is guaranteed to be smaller than or equal to the `max_item_count`.
    count: usize,
}

#[allow(dead_code)]
impl ItemSlot {
    /// Create an empty slot for the given item.
    pub(crate) fn new(item_id: ItemId, max_item_count: usize) -> Self {
        Self {
            item_id,
            max_item_count,
            count: 0,
        }
    }

    /// Create a slot fro the given item with the given count.
    ///
    /// # Panics
    ///
    /// It must be `count <= max_item_count` or this function will panic.
    #[cfg(test)]
    pub(crate) fn new_with_count(item_id: ItemId, max_item_count: usize, count: usize) -> Self {
        assert!(count <= max_item_count);

        Self {
            item_id,
            max_item_count,
            count,
        }
    }

    /// The unique identifier of the item in the slot.
    pub(crate) fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    /// The number of items in this slot.
    pub(crate) fn count(&self) -> usize {
        self.count
    }

    /// The maximum number of items that can fit in this slot.
    pub(crate) fn max_item_count(&self) -> usize {
        self.max_item_count
    }

    /// The number of items that can still fit in the item slot.
    pub(crate) fn remaining_space(&self) -> usize {
        self.max_item_count - self.count
    }

    /// Returns `true` if there are no items stored in this slot.
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns `true` if the maximum item count of this slot has been reached.
    pub(crate) fn is_full(&self) -> bool {
        self.count == self.max_item_count
    }

    /// Determine if this slot can hold items of the given type.
    pub(crate) fn is_for_item(&self, item_id: &ItemId) -> bool {
        self.item_id == *item_id
    }

    /// Try to add as many items to the inventory as possible, up to the given count.
    ///
    /// - If all items can fit in the slot, they are all added and `Ok` is returned.
    /// - Otherwise, all items that can fit are added and `Err` is returned.
    pub(crate) fn add_until_full(&mut self, count: usize) -> Result<(), AddOneItemError> {
        let new_count = self.count + count;

        if new_count > self.max_item_count {
            self.count = self.max_item_count;

            Err(AddOneItemError {
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
    pub(crate) fn add_all_or_nothing(&mut self, count: usize) -> Result<(), AddOneItemError> {
        if self.remaining_space() < count {
            Err(AddOneItemError {
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
    pub(crate) fn remove_until_empty(&mut self, count: usize) -> Result<(), RemoveOneItemError> {
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
    pub(crate) fn remove_all_or_nothing(&mut self, count: usize) -> Result<(), RemoveOneItemError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_item_type_count_and_capacity_for_empty_slot() {
        let item_slot = ItemSlot::new(ItemId::acacia_leaf(), 10);

        assert_eq!(format!("{item_slot}"), "acacia_leaf (0/10)".to_string());
    }

    #[test]
    fn should_display_item_type_count_and_capacity_for_filled_slot() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 6,
        };

        assert_eq!(format!("{item_slot}"), "acacia_leaf (6/10)".to_string());
    }

    #[test]
    fn should_be_empty_when_count_is_0() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 0,
        };

        assert!(item_slot.is_empty());
    }

    #[test]
    fn should_not_be_empty_when_count_is_1() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 1,
        };

        assert!(!item_slot.is_empty());
    }

    #[test]
    fn should_be_full_when_count_is_capacity() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 10,
        };

        assert!(item_slot.is_full());
    }

    #[test]
    fn should_not_be_full_when_count_is_less_than_capacity() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 9,
        };

        assert!(!item_slot.is_full());
    }

    #[test]
    fn should_calculate_remaining_space_for_empty_slot() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 0,
        };

        assert_eq!(item_slot.remaining_space(), 10);
    }

    #[test]
    fn should_calculate_remaining_space_for_half_full_slot() {
        let item_slot = ItemSlot {
            item_id: ItemId::acacia_leaf(),
            max_item_count: 10,
            count: 5,
        };

        assert_eq!(item_slot.remaining_space(), 5);
    }

    mod add {
        mod until_full {
            use super::super::*;

            #[test]
            fn should_be_ok_when_all_fit() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 0,
                };

                assert_eq!(item_slot.add_until_full(10), Ok(()));
                assert_eq!(item_slot.count(), 10);
            }

            #[test]
            fn should_fill_up_when_not_all_fit() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 5,
                };

                assert_eq!(
                    item_slot.add_until_full(10),
                    Err(AddOneItemError { excess_count: 5 })
                );
                assert_eq!(item_slot.count(), 10);
            }
        }

        mod all_or_nothing {
            use super::super::*;

            #[test]
            fn should_be_ok_when_all_fit() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 0,
                };

                assert_eq!(item_slot.add_all_or_nothing(10), Ok(()));
                assert_eq!(item_slot.count(), 10);
            }

            #[test]
            fn should_not_add_anything_if_not_all_fit() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 5,
                };

                assert_eq!(
                    item_slot.add_all_or_nothing(10),
                    Err(AddOneItemError { excess_count: 5 })
                );
                assert_eq!(item_slot.count(), 5);
            }
        }
    }

    mod remove {
        mod until_empty {
            use super::super::*;

            #[test]
            fn should_be_ok_when_all_exist() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 10,
                };

                assert_eq!(item_slot.remove_until_empty(10), Ok(()));
                assert_eq!(item_slot.count(), 0);
            }

            #[test]
            fn should_empty_if_not_all_exist() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 5,
                };

                assert_eq!(
                    item_slot.remove_until_empty(10),
                    Err(RemoveOneItemError { missing_count: 5 })
                );
                assert_eq!(item_slot.count(), 0);
            }
        }

        mod all_or_nothing {
            use super::super::*;

            #[test]
            fn should_be_ok_when_all_exist() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 10,
                };

                assert_eq!(item_slot.remove_all_or_nothing(10), Ok(()));
                assert_eq!(item_slot.count(), 0);
            }

            #[test]
            fn should_not_remove_anything_if_not_all_exist() {
                let mut item_slot = ItemSlot {
                    item_id: ItemId::acacia_leaf(),
                    max_item_count: 10,
                    count: 5,
                };

                assert_eq!(
                    item_slot.remove_all_or_nothing(10),
                    Err(RemoveOneItemError { missing_count: 5 })
                );
                assert_eq!(item_slot.count(), 5);
            }
        }
    }
}
