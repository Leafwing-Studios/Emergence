//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::{
    items::{inventory::Inventory, slot::ItemSlot, ItemCount, ItemId},
    structures::crafting::{InputInventory, OutputInventory},
};

use super::behavior::{CurrentAction, UnitAction};

/// The item(s) that a unit is carrying.
#[derive(Component, Debug, Deref, DerefMut)]
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
    #[allow(dead_code)]
    pub(crate) fn count(&self) -> usize {
        if let Some(item_slot) = self.item_slot() {
            item_slot.count()
        } else {
            0
        }
    }
}

pub(super) fn pickup_and_drop_items(
    mut unit_query: Query<(&mut CurrentAction, &mut HeldItem)>,
    mut input_query: Query<&mut InputInventory>,
    mut output_query: Query<&mut OutputInventory>,
) {
    for (mut current_action, mut held_item) in unit_query.iter_mut() {
        if current_action.finished() {
            let action = current_action.action().clone();
            let mut should_idle = false;

            if let UnitAction::PickUp {
                item_id,
                output_entity,
            } = action
            {
                if let Ok(mut output_inventory) = output_query.get_mut(*output_entity) {
                    let item_count = ItemCount::new(item_id.clone(), 1);
                    if let Ok(removed_items) = output_inventory.try_remove_item(&item_count) {
                        // Transfer the items
                        todo!()
                    } else {
                        // Inventory was empty
                        should_idle = true;
                    }
                } else {
                    // Something has gone wrong (like the structure was despawned), just idle.
                    should_idle = true;
                }
            }

            if let UnitAction::DropOff {
                item_id,
                input_entity,
            } = action
            {
                if let Ok(mut input_inventory) = input_query.get_mut(*input_entity) {
                    let item_count = ItemCount::new(item_id.clone(), 1);
                    if let Ok(removed_items) = held_item.try_remove_item(&item_count) {
                        // Transfer the items
                        todo!()
                    } else {
                        // Inventory was empty
                        should_idle = true;
                    }
                } else {
                    // Something has gone wrong (like the structure was despawned), just idle.
                    should_idle = true;
                }
            }

            if should_idle {
                *current_action = CurrentAction::idle();
            }
        }
    }
}
