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
    mut current_litter_piles: Local<HashMap<TilePos, Entity>>,
    terrain_query: Query<(&TilePos, Ref<StorageInventory>), With<Id<Terrain>>>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (&tile_pos, storage_inventory) in terrain_query.iter() {
        if storage_inventory.is_changed() {
            // Clean up any old models
            if let Some(&entity) = current_litter_piles.get(&tile_pos) {
                commands.entity(entity).despawn_recursive();
                current_litter_piles.remove(&tile_pos);
            }

            let inventory_state = storage_inventory.state();
            // Don't spawn anything if there's no litter.
            if inventory_state != InventoryState::Empty {
                let scene_handle = terrain_handles.litter_models.get(&inventory_state).unwrap();
                let litter_entity = commands
                    .spawn(SceneBundle {
                        scene: scene_handle.clone(),
                        transform: Transform::from_translation(
                            tile_pos.into_world_pos(&map_geometry),
                        ),
                        ..Default::default()
                    })
                    .id();
                current_litter_piles.insert(tile_pos, litter_entity);
            }
        }
    }
}
