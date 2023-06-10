//! Graphics and animation code for piles of junk, floating or otherwise.

use crate::{
    geometry::{MapGeometry, VoxelPos},
    litter::{Floating, Litter},
    terrain::terrain_assets::TerrainHandles,
    water::WaterDepth,
};
use bevy::prelude::*;

/// Spawn and despawn litter scenes based on the items stored as litter on each tile.
pub(super) fn render_litter_piles(
    terrain_handles: Res<TerrainHandles>,
    litter_query: Query<(Entity, &VoxelPos, Ref<Litter>, &mut Transform, &Floating)>,
    water_height_query: Query<(&WaterDepth, &VoxelPos)>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
) {
    // TODO: set the model based on inventory contents.

    // TODO: adjust the transform based on the water height.
}

/// Computes the [`Transform`] for a floating litter entity.
fn floating_litter_transform(
    voxel_pos: VoxelPos,
    water_height_query: &Query<(&WaterDepth, &VoxelPos)>,
    map_geometry: &MapGeometry,
) -> Result<Transform, ()> {
    let mut transform = Transform::from_translation(voxel_pos.into_world_pos(map_geometry));
    let Ok(terrain_entity) = map_geometry.get_terrain(voxel_pos.hex) else {
        return Err(());
    };
    let (water_depth, &terrain_pos) = water_height_query.get(terrain_entity).unwrap();
    let desired_height = water_depth.surface_height(terrain_pos.height());

    transform.translation.y = desired_height.into_world_pos();
    Ok(transform)
}
