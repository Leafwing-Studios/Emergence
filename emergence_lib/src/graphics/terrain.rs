use crate::{
    asset_management::manifest::Id,
    crafting::components::StorageInventory,
    items::inventory::InventoryState,
    simulation::geometry::{MapGeometry, TilePos},
    terrain::{terrain_assets::TerrainHandles, terrain_manifest::Terrain},
};
use bevy::{prelude::*, utils::HashMap};

/// Graphics and animation code for terrain.

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn manage_litter_piles(
    terrain_handles: Res<TerrainHandles>,
    // A simple cache of the current litter piles.
    mut current_litter_piles: Local<HashMap<TilePos, (InventoryState, Entity)>>,
    terrain_query: Query<(&TilePos, Ref<StorageInventory>), With<Id<Terrain>>>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (&tile_pos, storage_inventory) in terrain_query.iter() {
        if !storage_inventory.is_changed() {
            continue;
        }

        let current_inventory_state = storage_inventory.state();

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
                transform: Transform::from_translation(tile_pos.into_world_pos(&map_geometry)),
                ..Default::default()
            })
            .id();
        current_litter_piles.insert(tile_pos, (current_inventory_state, litter_entity));
    }
}
