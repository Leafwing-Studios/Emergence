//! Graphics and animation code for terrain.

use crate::{
    geometry::{Height, MapGeometry, VoxelPos},
    items::inventory::InventoryState,
    terrain::{litter::Litter, terrain_assets::TerrainHandles},
    water::WaterDepth,
};
use bevy::{prelude::*, utils::HashMap};

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn render_litter_piles(
    terrain_handles: Res<TerrainHandles>,
    // A simple cache of the current litter piles.
    mut current_ground_litter_piles: Local<HashMap<VoxelPos, (InventoryState, Entity)>>,
    mut current_floating_litter_piles: Local<HashMap<VoxelPos, (InventoryState, Entity)>>,
    terrain_query: Query<(Entity, &VoxelPos, Ref<Litter>)>,
    water_height_query: Query<(&WaterDepth, &Height)>,
    // PERF: we could add a marker component to improve parallelism
    mut floating_litter_query: Query<&mut Transform>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, &voxel_pos, litter) in terrain_query.iter() {
        let current_ground_inventory_state = litter.on_ground.state();
        let current_floating_inventory_state = litter.floating.state();

        // Clean up any old models
        if let Some((previous_inventory_state, entity)) =
            current_ground_litter_piles.get(&voxel_pos)
        {
            // Only despawn if the inventory state has changed.
            if *previous_inventory_state != current_ground_inventory_state {
                if let Some(entity_commands) = commands.get_entity(*entity) {
                    entity_commands.despawn_recursive();
                }
                current_ground_litter_piles.remove(&voxel_pos);
            } else {
                continue;
            }
        }

        if let Some((previous_inventory_state, entity)) =
            current_floating_litter_piles.get(&voxel_pos)
        {
            // Only despawn if the inventory state has changed.
            if *previous_inventory_state != current_floating_inventory_state {
                if let Some(entity_commands) = commands.get_entity(*entity) {
                    entity_commands.despawn_recursive();
                }
                current_floating_litter_piles.remove(&voxel_pos);
            } else {
                continue;
            }
        }

        // Spawn ground litter
        if current_ground_inventory_state != InventoryState::Empty {
            let scene_handle = terrain_handles
                .litter_models
                .get(&current_ground_inventory_state)
                .unwrap();
            let litter_entity = commands
                .spawn(SceneBundle {
                    scene: scene_handle.clone(),
                    ..Default::default()
                })
                .id();
            commands.entity(terrain_entity).add_child(litter_entity);
            current_ground_litter_piles
                .insert(voxel_pos, (current_ground_inventory_state, litter_entity));
        }

        // Spawn floating litter
        if current_floating_inventory_state != InventoryState::Empty {
            let scene_handle = terrain_handles
                .litter_models
                .get(&current_floating_inventory_state)
                .unwrap();
            let litter_entity = commands
                .spawn(SceneBundle {
                    scene: scene_handle.clone(),
                    // This can't be a child of the terrain entity because it needs to be able to
                    // change heights with the water.
                    transform: floating_litter_transform(
                        voxel_pos,
                        &water_height_query,
                        &map_geometry,
                    )
                    .unwrap_or_default(),
                    ..Default::default()
                })
                .id();
            current_floating_litter_piles
                .insert(voxel_pos, (current_floating_inventory_state, litter_entity));
        }
    }

    // Update the height of floating litter
    for (voxel_pos, (_, entity)) in current_floating_litter_piles.iter() {
        if let Ok(mut transform) = floating_litter_query.get_mut(*entity) {
            if let Ok(new_transform) =
                floating_litter_transform(*voxel_pos, &water_height_query, &map_geometry)
            {
                *transform = new_transform;
            } else {
                warn!("Tried to spawn floating litter on dry land");
            }
        }
    }
}

/// Computes the [`Transform`] for a floating litter entity.
fn floating_litter_transform(
    voxel_pos: VoxelPos,
    water_height_query: &Query<(&WaterDepth, &Height)>,
    map_geometry: &MapGeometry,
) -> Result<Transform, ()> {
    let mut transform = Transform::from_translation(voxel_pos.into_world_pos(map_geometry));
    let Some(terrain_entity) = map_geometry.get_terrain(voxel_pos) else {
        return Err(());
    };
    let (water_depth, &terrain_height) = water_height_query.get(terrain_entity).unwrap();
    let desired_height = water_depth.surface_height(terrain_height);

    transform.translation.y = desired_height.into_world_pos();
    Ok(transform)
}
