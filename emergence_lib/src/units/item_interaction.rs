//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{Id, Item},
    items::{inventory::Inventory, slot::ItemSlot},
};
use core::fmt::Display;

/// The item(s) that a unit is carrying.
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(crate) struct HeldItem {
    /// The internal representation.
    pub(crate) inventory: Inventory,
}

impl Default for HeldItem {
    fn default() -> Self {
        HeldItem {
            inventory: Inventory::new(1),
        }
    }
}

impl Display for HeldItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(slot) = self.item_slot() {
            let item = slot.item_id();
            let count = slot.count();
            write!(f, "{count} {item}")
        } else {
            write!(f, "Empty")
        }
    }
}

impl HeldItem {
    /// The item and quantity held, if any.
    pub(crate) fn item_slot(&self) -> Option<&ItemSlot> {
        self.inventory.iter().next()
    }

    /// The type of item held.
    pub(crate) fn item_id(&self) -> Option<Id<Item>> {
        let item_slot = self.item_slot()?;
        Some(item_slot.item_id())
    }
}

/// Clears out any slots that no longer have items in them.
pub(super) fn clear_empty_slots(mut query: Query<&mut HeldItem>) {
    for mut held_item in query.iter_mut() {
        held_item.clear_empty_slots()
    }
}
