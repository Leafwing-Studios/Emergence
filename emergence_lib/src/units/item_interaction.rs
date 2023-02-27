//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::{
    items::{inventory::Inventory, slot::ItemSlot, ItemCount, ItemId, ItemManifest},
    structures::crafting::{InputInventory, OutputInventory},
};
use core::fmt::Display;

use super::{
    actions::{CurrentAction, UnitAction},
    goals::Goal,
};

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
    pub(crate) fn item_id(&self) -> Option<ItemId> {
        let item_slot = self.item_slot()?;
        Some(item_slot.item_id())
    }
}

/// A system which performs the transfer of items between units and structures.
pub(super) fn pickup_and_drop_items(
    mut unit_query: Query<(&CurrentAction, &mut Goal, &mut HeldItem)>,
    mut input_query: Query<&mut InputInventory>,
    mut output_query: Query<&mut OutputInventory>,
    item_manifest: Res<ItemManifest>,
) {
    let item_manifest = &*item_manifest;

    for (current_action, mut current_goal, mut held_item) in unit_query.iter_mut() {
        if current_action.finished() {
            if let UnitAction::PickUp {
                item_id,
                output_entity,
            } = current_action.action()
            {
                if let Ok(mut output_inventory) = output_query.get_mut(*output_entity) {
                    // Transfer one item at a time
                    let item_count = ItemCount::new(*item_id, 1);
                    let _transfer_result = output_inventory.transfer_item(
                        &item_count,
                        &mut held_item.inventory,
                        item_manifest,
                    );

                    // If our unit's all loaded, swap to delivering it
                    *current_goal = if held_item.is_full() {
                        Goal::DropOff(*item_id)
                    // If we can carry more, try and grab more items
                    } else {
                        Goal::Pickup(*item_id)
                    }
                } else {
                    // Something has gone wrong (like the structure was despawned)
                    *current_goal = Goal::Wander
                }
            } else if let UnitAction::DropOff {
                item_id,
                input_entity,
            } = current_action.action()
            {
                if let Ok(mut input_inventory) = input_query.get_mut(*input_entity) {
                    // Transfer one item at a time
                    let item_count = ItemCount::new(*item_id, 1);
                    let _transfer_result = held_item.transfer_item(
                        &item_count,
                        &mut input_inventory.inventory,
                        item_manifest,
                    );

                    // If our unit is unloaded, swap to wandering to find something else to do
                    *current_goal = if held_item.is_empty() {
                        Goal::Wander
                    // If we still have items, keep unloading
                    } else {
                        Goal::DropOff(*item_id)
                    }
                } else {
                    // Something has gone wrong (like the structure was despawned)
                    *current_goal = Goal::Wander
                }
            } else if let UnitAction::Abandon = current_action.action() {
                // TODO: actually put these dropped items somewhere
                *held_item = HeldItem::default();
            } else {
                // Other actions are not handled in this system
                return;
            };
        }
    }
}

/// Clears out any slots that no longer have items in them.
pub(super) fn clear_empty_slots(mut query: Query<&mut HeldItem>) {
    for mut held_item in query.iter_mut() {
        held_item.clear_empty_slots()
    }
}
