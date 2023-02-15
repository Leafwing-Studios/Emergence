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
            *tile_pos = *target_tile;

            let pos = map_geometry.layout.hex_to_world_pos(target_tile.hex);
            let terrain_height = *map_geometry.height_index.get(&target_tile).unwrap();

            *transform = Transform::from_xyz(pos.x, terrain_height, pos.y);
        }
    }
}
