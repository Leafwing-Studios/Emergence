use bevy::prelude::*;

use crate::simulation::geometry::MapGeometry;
use crate::simulation::geometry::TilePos;

use super::{behavior::events::MoveThisTurn, UnitId};

pub(super) fn move_unit_to_tile(
    mut move_events: EventReader<MoveThisTurn>,
    mut unit_query: Query<(&mut Transform, &mut TilePos), With<UnitId>>,
    map_geometry: Res<MapGeometry>,
) {
    for event in move_events.iter() {
        // FIXME: this should probably be resolved with more coherent scheduling
        // The unit may exist, but may not be populated yet.
        if let Ok((mut unit_transform, mut unit_tile_pos)) = unit_query.get_mut(event.unit_entity) {
            let pos = map_geometry.layout.hex_to_world_pos(event.target_tile.hex);
            let terrain_height = *map_geometry.height_index.get(&event.target_tile).unwrap();

            *unit_transform = Transform::from_xyz(pos.x, terrain_height, pos.y);
            *unit_tile_pos = event.target_tile;
        }
    }
}
