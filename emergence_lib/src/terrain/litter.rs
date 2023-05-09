//! Logic and components for littered items.

use bevy::prelude::*;
use bevy::utils::Duration;
use hexx::Direction;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::{
    asset_management::manifest::Id,
    crafting::{
        inventories::StorageInventory,
        item_tags::{ItemKind, ItemTag},
    },
    items::{
        errors::RemoveOneItemError,
        item_manifest::{Item, ItemManifest},
        ItemCount,
    },
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{direction_from_angle, MapGeometry, TilePos},
    water::{WaterDepth, WaterTable},
};

/// Items that are littered without a container.
///
/// This component is tracked on a per-tile basis.
#[derive(Component, Clone, Debug)]
pub(crate) struct Litter {
    /// The items that are littered on the ground.
    pub(crate) on_ground: StorageInventory,
    /// The items that are floating on the water.
    pub(crate) floating: StorageInventory,
}

impl Litter {
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

/// Make litter in tiles submerged by water float (and stop it from floating when there's no water).
pub(super) fn make_litter_float(
    mut query: Query<(&TilePos, &mut Litter)>,
    water_table: Res<WaterTable>,
    item_manifest: Res<ItemManifest>,
) {
    for (&tile_pos, mut litter) in query.iter_mut() {
        if let WaterDepth::Flooded(..) = water_table.water_depth(tile_pos) {
            // PERF: this clone is probably not needed, but it helps deal with the borrow checker
            // It's fine to iterate over a cloned list of items, because we're only moving them out of the list one at a time
            for item_on_ground in litter.on_ground.clone().iter() {
                let item_id = item_on_ground.item_id();

                // Check that the item floats
                if !item_manifest.has_tag(item_id, ItemTag::Buoyant) {
                    continue;
                }

                // Try to transfer as many items as possible to the floating inventory
                let item_count = ItemCount {
                    item_id: item_on_ground.item_id(),
                    count: item_on_ground.count(),
                };

                // PERF: we could use mem::swap plus a local to avoid the clone
                // Do the hokey-pokey to get around the borrow checker
                let mut on_ground = litter.on_ground.clone();

                // We don't care how much was transferred; failing to transfer is fine
                let _ = on_ground.transfer_item(&item_count, &mut litter.floating, &item_manifest);
                litter.on_ground = on_ground;
            }
        } else {
            for floating_item in litter.floating.clone().iter() {
                // Try to transfer as many items as possible to the ground inventory
                let item_count = ItemCount {
                    item_id: floating_item.item_id(),
                    count: floating_item.count(),
                };

                // PERF: we could use mem::swap plus a local to avoid the clone
                // Do the hokey-pokey to get around the borrow checker
                let mut floating = litter.floating.clone();

                // We don't care how much was transferred; failing to transfer is fine
                let _ = floating.transfer_item(&item_count, &mut litter.on_ground, &item_manifest);
                litter.floating = floating;
            }
        }
    }
}

/// The direction and time remaining for a piece of litter to drift with the current.
#[derive(Component, Default)]
pub(super) struct LitterDrift {
    /// The direction the litter is drifting.
    pub(super) direction: Option<Direction>,
    /// The time remaining for the litter to drift.
    pub(super) timer: Timer,
}

impl LitterDrift {
    /// Starts the timer for the litter to drift with the current.
    pub(super) fn start(&mut self, direction: Direction, required_time: Duration) {
        // This attempts to avoid allocations by reusing the timer
        self.direction = Some(direction);
        self.timer.set_duration(required_time);
        self.timer.reset();
    }

    /// Ends the timer for the litter to drift with the current.
    pub(super) fn finish(&mut self) {
        self.direction = None;
        // The timer is reset when starting, so we don't need to do anything here
    }
}

/// Carries floating litter along with the surface water current.
pub(super) fn carry_floating_litter_with_current(
    mut litter_query: Query<(Entity, &TilePos, &mut Litter, &mut LitterDrift)>,
    fixed_time: Res<FixedTime>,
    water_table: Res<WaterTable>,
    item_manifest: Res<ItemManifest>,
    map_geometry: Res<MapGeometry>,
) {
    /// Controls how fast litter drifts with the current
    ///
    /// Higher values mean litter drifts faster.
    /// This must be greater than 0.
    const ITEM_DRIFT_RATE: f32 = 1e-4;

    /// Controls how much litter varies relative to the current direction
    ///
    /// This is the standard deviation of the normal distribution used to determine the drift angle, and is in units of radians.
    /// This must be greater than 0.
    const DRIFT_DEVIATION: f32 = 1.0;

    let delta_time = fixed_time.period;
    // By collecting a list of (source, destination) pairs, we avoid borrowing the litter query twice,
    // sparing us from the wrath of the borrow checker
    let mut proposed_transfers: Vec<(Entity, TilePos)> = Vec::new();

    let rng = &mut thread_rng();
    let normal_distribution = Normal::new(0.0, DRIFT_DEVIATION).unwrap();

    for (source_entity, &tile_pos, litter, mut litter_drift) in litter_query.iter_mut() {
        if let WaterDepth::Flooded(water_depth) = water_table.water_depth(tile_pos) {
            litter_drift.timer.tick(delta_time);

            // Don't both computing drift if there's nothing floating
            if litter.floating.is_empty() {
                continue;
            }

            let flow_velocity = water_table.flow_velocity(tile_pos);
            let flow_direction = flow_velocity.direction() + normal_distribution.sample(rng);

            // Volume transferred = cross-sectional area * water speed * time
            // Cross-sectional area = water depth * tile area
            // Volume transferred = water depth * tile area * water speed * time
            // Water speed = volume transferred / time * tile area / water depth
            // Tile area is 1 (because we're computing on a per-tile basis), and flow_velocity is in tiles per second
            // Therefore: water speed = flow velocity / water depth
            let water_speed = flow_velocity.magnitude().0 / water_depth.0;

            // If the litter is not already drifting, start it drifting
            if litter_drift.direction.is_none() {
                let direction =
                    direction_from_angle(flow_direction, map_geometry.layout.orientation);
                let time_to_drift = water_speed * delta_time.as_secs_f32() / ITEM_DRIFT_RATE;

                litter_drift.start(direction, Duration::from_secs_f32(time_to_drift));
            }

            // If the litter has finished drifting, stop it drifting and move it
            if litter_drift.timer.finished() {
                if let Some(direction) = litter_drift.direction {
                    let new_position = tile_pos.neighbor(direction);
                    // Record that this change needs to be tried
                    proposed_transfers.push((source_entity, new_position));
                }

                litter_drift.finish();
            }
        } else {
            // PERF: we could probably avoid doing any work in this branch by more carefully tracking when things start floating
            litter_drift.finish();
        }
    }

    for (source_entity, new_position) in proposed_transfers.iter() {
        let Some(target_entity) = map_geometry.get_terrain(*new_position) else {
            continue;
        };
        let [source_query_item, target_query_item] = litter_query
            .get_many_mut([*source_entity, target_entity])
            .unwrap();

        let mut source_litter: Mut<Litter> = source_query_item.2;
        let mut target_litter: Mut<Litter> = target_query_item.2;

        // Do the hokey-pokey to get around the borrow checker
        let mut source_floating = source_litter.floating.clone();
        let mut target_floating = target_litter.floating.clone();

        // We don't care how much was transferred; failing to transfer is fine
        let result = source_floating.transfer_all(&mut target_floating, &item_manifest);

        // If the transfer was successful, update the litter
        if result.is_ok() {
            source_litter.floating = source_floating;
            target_litter.floating = target_floating;
        }
    }
}
