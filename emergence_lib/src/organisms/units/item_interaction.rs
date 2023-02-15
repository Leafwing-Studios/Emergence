//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::items::{inventory::Inventory, slot::ItemSlot, ItemId};

/// The item(s) that a unit is carrying.
#[derive(Component, Debug)]
pub(crate) struct HeldItem {
    /// The internal representation.
    inventory: Inventory,
}

impl Default for HeldItem {
    fn default() -> Self {
        HeldItem {
            inventory: Inventory::new(1),
        }
    }
}

impl HeldItem {
    /// The item and quantity held, if any.
    pub(crate) fn item_slot(&self) -> Option<&ItemSlot> {
        self.inventory.iter().next()
    }

    /// The type of item that is being held, if any.
    pub(crate) fn item_id(&self) -> Option<&ItemId> {
        let item_slot = self.item_slot()?;
        Some(item_slot.item_id())
    }

    /// The number of items of a single type being held.
    pub(crate) fn count(&self) -> usize {
        if let Some(item_slot) = self.item_slot() {
            item_slot.count()
        } else {
            0
        }
    }
}
