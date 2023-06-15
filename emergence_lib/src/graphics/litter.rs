//! Graphics and animation code for piles of junk, floating or otherwise.

use crate::{
    geometry::{MapGeometry, VoxelPos},
    water::WaterDepth,
};
use bevy::prelude::*;

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn render_litter_piles() {}

/// Computes the [`Transform`] for a floating litter entity.
#[allow(dead_code)]
fn floating_litter_transform(
    voxel_pos: VoxelPos,
    water_height_query: &Query<(&WaterDepth, &VoxelPos)>,
    map_geometry: &MapGeometry,
) -> Result<Transform, ()> {
    let mut transform = Transform::from_translation(voxel_pos.into_world_pos());
    let Ok(terrain_entity) = map_geometry.get_terrain(voxel_pos.hex) else {
        return Err(());
    };
    let (water_depth, &terrain_pos) = water_height_query.get(terrain_entity).unwrap();
    let desired_height = water_depth.surface_height(terrain_pos.height());

    transform.translation.y = desired_height.into_world_pos();
    Ok(transform)
}
