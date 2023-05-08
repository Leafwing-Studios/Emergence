//! Graphics and animation code for terrain.

use crate::{
    items::inventory::InventoryState,
    simulation::geometry::TilePos,
    terrain::{litter::Litter, terrain_assets::TerrainHandles},
};
use bevy::{prelude::*, utils::HashMap};

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn manage_litter_piles(
    terrain_handles: Res<TerrainHandles>,
    // A simple cache of the current litter piles.
    mut current_litter_piles: Local<HashMap<TilePos, (InventoryState, Entity)>>,
    terrain_query: Query<(Entity, &TilePos, Ref<Litter>)>,
    mut commands: Commands,
) {
    for (terrain_entity, &tile_pos, litter) in terrain_query.iter() {
        if !litter.is_changed() {
            continue;
        }

        // TODO: also draw floating litter piles
        let current_inventory_state = litter.on_ground().state();

        // Clean up any old models
        if let Some((previous_inventory_state, entity)) = current_litter_piles.get(&tile_pos) {
            // Only despawn if the inventory state has changed.
            if *previous_inventory_state != current_inventory_state {
                commands.entity(*entity).despawn_recursive();
                current_litter_piles.remove(&tile_pos);
            } else {
                continue;
            }
        }

        // Don't spawn anything if there's no litter.
        if current_inventory_state == InventoryState::Empty {
            continue;
        }

        let scene_handle = terrain_handles
            .litter_models
            .get(&current_inventory_state)
            .unwrap();
        let litter_entity = commands
            .spawn(SceneBundle {
                scene: scene_handle.clone(),
                ..Default::default()
            })
            .id();
        commands.entity(terrain_entity).add_child(litter_entity);
        current_litter_piles.insert(tile_pos, (current_inventory_state, litter_entity));
    }
}
