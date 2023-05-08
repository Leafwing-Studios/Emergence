//! Logic and components for littered items.

use bevy::prelude::*;

use crate::{
    crafting::{inventories::StorageInventory, item_tags::ItemKind},
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
