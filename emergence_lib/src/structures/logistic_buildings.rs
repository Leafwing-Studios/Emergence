//! Logic for buildings that move items around.

use bevy::prelude::*;

use crate::{
    crafting::{inventories::InputInventory, item_tags::ItemKind, recipe::RecipeInput},
    items::item_manifest::ItemManifest,
    signals::{Emitter, SignalStrength, SignalType},
    simulation::{
        geometry::{Facing, MapGeometry, TilePos},
        SimulationSet,
    },
    terrain::litter::Litter,
};

/// A building that spits out items.
#[derive(Component)]
pub(super) struct ReleasesItems;

/// A building that takes in items.
#[derive(Component)]
pub(super) struct AbsorbsItems;

/// Logic that controls how items are moved around by structures.
pub(super) struct LogisticsPlugin;

impl Plugin for LogisticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (release_items, logistic_buildings_signals)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// Causes buildings that emit items to place them in the litter in front of them.
fn release_items(
    mut structure_query: Query<(&TilePos, &Facing, &mut InputInventory), With<ReleasesItems>>,
    mut litter_query: Query<&mut Litter>,
    item_manifest: Res<ItemManifest>,
    map_geometry: Res<MapGeometry>,
) {
    for (structure_pos, structure_facing, mut input_inventory) in structure_query.iter_mut() {
        let tile_pos = structure_pos.neighbor(structure_facing.direction);

        let litter_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let mut litter = litter_query.get_mut(litter_entity).unwrap();

        // Do the hokey-pokey to get around the borrow checker
        let mut source = input_inventory.clone();
        let mut target = litter.on_ground.clone();

        let result = source
            .inventory_mut()
            .transfer_all(&mut target, &item_manifest);

        // If the transfer was successful, update the inventories
        if result.is_ok() {
            for item_slot in source.iter() {
                let item_count = item_slot.item_count();
                let recipe_input = RecipeInput::Exact(vec![item_count]);

                input_inventory
                    .consume_items(&recipe_input, &item_manifest)
                    .unwrap();
            }

            litter.on_ground = target;
        }
    }
}

fn logistic_buildings_signals(
    mut release_query: Query<(&mut Emitter, &mut InputInventory), With<ReleasesItems>>,
) {
    /// Controls how strong the signal is for logistic buildings.
    const LOGISTIC_SIGNAL_STRENGTH: f32 = 10.;

    let signal_strength = SignalStrength::new(LOGISTIC_SIGNAL_STRENGTH);

    for (mut emitter, input_inventory) in release_query.iter_mut() {
        emitter.signals.clear();
        for item_slot in input_inventory.iter() {
            if !item_slot.is_full() {
                let item_kind = match *input_inventory {
                    InputInventory::Exact { .. } => ItemKind::Single(item_slot.item_id()),
                    InputInventory::Tagged { tag, .. } => ItemKind::Tag(tag),
                };

                // This should be a Pull signal, rather than a Stores signal to
                // ensure that goods can be continuously harvested and shipped.
                let signal_type: SignalType = SignalType::Pull(item_kind);
                emitter.signals.push((signal_type, signal_strength));
            }
        }
    }
}
