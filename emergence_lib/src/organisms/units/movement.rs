use bevy::prelude::*;

use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

use super::behavior::{CurrentAction, UnitAction};
use super::UnitId;

pub(super) fn move_unit_to_tile(
    mut unit_query: Query<(&mut Transform, &mut TilePos, &CurrentAction), With<UnitId>>,
    map_geometry: Res<MapGeometry>,
) {
    for (mut transform, mut tile_pos, current_action) in unit_query.iter_mut() {
        if let UnitAction::Move(target_tile) = current_action.action() {
            if current_action.finished() {
                let direction = tile_pos.direction_to(**target_tile);
                let angle = direction.angle(&map_geometry.layout.orientation);

                transform.rotation = Quat::from_axis_angle(Vec3::Y, angle);

                let pos = map_geometry.layout.hex_to_world_pos(target_tile.hex);
                let terrain_height = *map_geometry.height_index.get(&target_tile).unwrap();

                transform.translation = Vec3 {
                    x: pos.x,
                    // Bevy is y-up
                    y: terrain_height,
                    z: pos.y,
                };

                *tile_pos = *target_tile;
            }
        }
    }
}
