//! Logic for buildings that move items around.

use bevy::prelude::*;

use crate::{
    crafting::inventories::InputInventory,
    items::item_manifest::ItemManifest,
    simulation::geometry::{Facing, MapGeometry, TilePos},
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
        app.add_system(emit_items);
    }
}

/// Causes buildings that emit items to place them in the litter in front of them.
fn emit_items(
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

                input_inventory
                    .fill_with_items(&item_count, &item_manifest)
                    .unwrap();
            }

            litter.on_ground = target;
        }
    }
}
