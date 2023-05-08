//! Logic and components for littered items.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    crafting::{inventories::StorageInventory, item_tags::ItemKind},
    items::{
        errors::RemoveOneItemError,
        item_manifest::{Item, ItemManifest},
        ItemCount,
    },
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{MapGeometry, TilePos},
};

/// Items that are littered without a container.
///
/// This component is tracked on a per-tile basis.
#[derive(Component, Clone, Debug)]
pub(crate) struct Litter {
    /// The items that are littered on the ground.
    on_ground: StorageInventory,
    /// The items that are floating on the water.
    floating: StorageInventory,
}

impl Litter {
    /// Returns a reference to the storage inventory for litter on the ground.
    pub(crate) fn on_ground(&self) -> &StorageInventory {
        &self.on_ground
    }

    /// Returns a mutable reference to the storage inventory for litter on the ground.
    pub(crate) fn on_ground_mut(&mut self) -> &mut StorageInventory {
        &mut self.on_ground
    }

    /// Returns a reference to the storage inventory for floating litter.
    pub(crate) fn floating(&self) -> &StorageInventory {
        &self.floating
    }

    /// Returns a mutable reference to the storage inventory for floating litter.
    pub(crate) fn floating_mut(&mut self) -> &mut StorageInventory {
        &mut self.floating
    }

    /// Does this inventory contain at least one matching item?
    pub(crate) fn contains_kind(&self, item_kind: ItemKind, item_manifest: &ItemManifest) -> bool {
        self.on_ground.contains_kind(item_kind, item_manifest)
            || self.floating.contains_kind(item_kind, item_manifest)
    }

    /// Does this litter currently have space for an item of this type?
    pub(crate) fn currently_accepts(
        &self,
        item_id: Id<Item>,
        item_manifest: &ItemManifest,
    ) -> bool {
        self.on_ground.currently_accepts(item_id, item_manifest)
    }

    /// Returns the first [`Id<Item>`] that matches the given [`ItemKind`], if any.
    ///
    /// Items on the ground will be checked first, then floating items.
    pub(crate) fn matching_item_id(
        &self,
        item_kind: ItemKind,
        item_manifest: &ItemManifest,
    ) -> Option<Id<Item>> {
        match self.on_ground.matching_item_id(item_kind, item_manifest) {
            Some(item_id) => Some(item_id),
            None => self.floating.matching_item_id(item_kind, item_manifest),
        }
    }

    /// Try to remove the given count of items from the inventory, together.
    ///
    /// Items on the ground will be checked first, then floating items.
    /// Items will never be drawn from both inventories simultaneously.
    pub(crate) fn remove_item_all_or_nothing(
        &mut self,
        item_count: &ItemCount,
    ) -> Result<(), RemoveOneItemError> {
        match self.on_ground.remove_item_all_or_nothing(item_count) {
            Ok(()) => Ok(()),
            Err(_) => self.floating.remove_item_all_or_nothing(item_count),
        }
    }

    /// The pretty formatting for the litter stored here.
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        let mut display = String::new();

        display.push_str("On Ground: ");
        for item_slot in self.on_ground.iter() {
            display.push_str(&item_slot.display(item_manifest));
            display.push_str(", ");
        }

        display.push_str("\nFloating: ");
        for item_slot in self.floating.iter() {
            display.push_str(&item_slot.display(item_manifest));
            display.push_str(", ");
        }

        display
    }
}

impl Default for Litter {
    fn default() -> Self {
        Litter {
            on_ground: StorageInventory::new(1, None),
            floating: StorageInventory::new(1, None),
        }
    }
}

/// Updates the signals produced by terrain tiles.
pub(super) fn set_terrain_emitters(mut query: Query<(&mut Emitter, Ref<Litter>)>) {
    for (mut emitter, litter) in query.iter_mut() {
        if litter.is_changed() {
            emitter.signals.clear();
            for item_slot in litter.on_ground.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match litter.on_ground.is_full() {
                    true => SignalType::Push(item_kind),
                    false => SignalType::Contains(item_kind),
                };
                let signal_strength = SignalStrength::new(10.);

                emitter.signals.push((signal_type, signal_strength));
            }

            for item_slot in litter.floating.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match litter.floating.is_full() {
                    true => SignalType::Push(item_kind),
                    false => SignalType::Contains(item_kind),
                };
                let signal_strength = SignalStrength::new(10.);

                emitter.signals.push((signal_type, signal_strength));
            }
        }
    }
}

/// The set of systems that update terrain emitters.
#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TerrainEmitters;

/// Tracks how much litter is on the ground on each tile.
pub(super) fn update_litter_index(
    query: Query<(&TilePos, &Litter), Changed<Litter>>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (&tile_pos, litter) in query.iter() {
        // Only litter on the ground is impassable.
        map_geometry.update_litter_state(tile_pos, litter.on_ground.state());
    }
}

/// The space in litter storage inventories is not reserved, so should be cleared
pub(super) fn clear_empty_litter(mut query: Query<&mut Litter>) {
    for mut litter in query.iter_mut() {
        litter.on_ground.clear_empty_slots();
        litter.floating.clear_empty_slots();
    }
}
