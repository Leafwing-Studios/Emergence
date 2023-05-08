//! Logic and components for littered items.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    crafting::{inventories::StorageInventory, item_tags::ItemKind},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{MapGeometry, TilePos},
};

use super::terrain_manifest::Terrain;

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

impl Default for Litter {
    fn default() -> Self {
        Litter {
            on_ground: StorageInventory::new(1, None),
            floating: StorageInventory::new(1, None),
        }
    }
}

/// Updates the signals produced by terrain tiles.
pub(super) fn set_terrain_emitters(
    mut query: Query<(&mut Emitter, Ref<StorageInventory>), With<Id<Terrain>>>,
) {
    for (mut emitter, storage_inventory) in query.iter_mut() {
        if storage_inventory.is_changed() {
            emitter.signals.clear();
            for item_slot in storage_inventory.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match storage_inventory.is_full() {
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
    query: Query<(&TilePos, &StorageInventory), (With<Id<Terrain>>, Changed<StorageInventory>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (&tile_pos, litter) in query.iter() {
        map_geometry.update_litter_state(tile_pos, litter.state());
    }
}
