//! Graphics and animation code for terrain.

use crate::{
    items::inventory::InventoryState,
    simulation::geometry::{MapGeometry, TilePos},
    terrain::{litter::Litter, terrain_assets::TerrainHandles},
    water::{WaterDepth, WaterTable},
};
use bevy::{prelude::*, utils::HashMap};

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn manage_litter_piles(
    terrain_handles: Res<TerrainHandles>,
    // A simple cache of the current litter piles.
    mut current_ground_litter_piles: Local<HashMap<TilePos, (InventoryState, Entity)>>,
    mut current_floating_litter_piles: Local<HashMap<TilePos, (InventoryState, Entity)>>,
    terrain_query: Query<(Entity, &TilePos, Ref<Litter>)>,
    // PERF: we could add a marker component to improve parallelism
    mut floating_litter_query: Query<&mut Transform>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, &tile_pos, litter) in terrain_query.iter() {
        if !litter.is_changed() {
            continue;
        }

        // TODO: also draw floating litter piles
        let current_ground_inventory_state = litter.on_ground.state();
        let current_floating_inventory_state = litter.floating.state();

        // Clean up any old models
        if let Some((previous_inventory_state, entity)) = current_ground_litter_piles.get(&tile_pos)
        {
            // Only despawn if the inventory state has changed.
            if *previous_inventory_state != current_ground_inventory_state {
                commands.entity(*entity).despawn_recursive();
                current_ground_litter_piles.remove(&tile_pos);
            } else {
                continue;
            }
        }

        if let Some((previous_inventory_state, entity)) =
            current_floating_litter_piles.get(&tile_pos)
        {
            // Only despawn if the inventory state has changed.
            if *previous_inventory_state != current_floating_inventory_state {
                commands.entity(*entity).despawn_recursive();
                current_ground_litter_piles.remove(&tile_pos);
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
                .insert(tile_pos, (current_ground_inventory_state, litter_entity));
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
                        tile_pos,
                        &WaterTable::default(),
                        &map_geometry,
                    )
                    .unwrap_or_default(),
                    ..Default::default()
                })
                .id();
            current_floating_litter_piles
                .insert(tile_pos, (current_floating_inventory_state, litter_entity));
        }
    }

    // Update the height of floating litter
    for (tile_pos, (_, entity)) in current_floating_litter_piles.iter() {
        if let Ok(mut transform) = floating_litter_query.get_mut(*entity) {
            if let Ok(new_transform) =
                floating_litter_transform(*tile_pos, &WaterTable::default(), &map_geometry)
            {
                *transform = new_transform;
            } else {
                warn!("Tried to spawn floating litter on dry land");
            }
        }
    }
}

fn floating_litter_transform(
    tile_pos: TilePos,
    water_table: &WaterTable,
    map_geometry: &MapGeometry,
) -> Result<Transform, ()> {
    let mut transform = Transform::from_translation(tile_pos.into_world_pos(&map_geometry));
    let terrain_height = map_geometry.get_height(tile_pos).unwrap();

    let desired_height = match water_table.water_depth(tile_pos) {
        WaterDepth::Dry | WaterDepth::Underground(_) => return Err(()),
        WaterDepth::Flooded(surface_water_depth) => terrain_height + surface_water_depth,
    };

    transform.translation.y = desired_height.into_world_pos();
    Ok(transform)
}
