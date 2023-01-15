//! A container of a single item type, without a capacity.

use std::fmt::Display;

use super::ItemId;

/// A specific amount of a given item.
#[derive(Debug, Clone)]
pub struct ItemCount {
    /// The unique identifier of the item being counted.
    item_id: ItemId,

    /// The number of items.
    count: usize,
}

impl ItemCount {
    /// Create a new item count with the given number of items.
    pub fn new(item_id: ItemId, count: usize) -> Self {
        Self { item_id, count }
    }

    /// A single one of the given item.
    pub fn one(item_id: ItemId) -> Self {
        Self { item_id, count: 1 }
    }

    /// The unique identifier of the item being counted.
    pub fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    /// The number of items.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Display for ItemCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.item_id, self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_item_type_and_count() {
        let item_count = ItemCount::new(ItemId::acacia_leaf(), 3);

        assert_eq!(format!("{item_count}"), "acacia_leaf (3)".to_string());
    }
}
