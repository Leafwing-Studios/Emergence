//! Graphics code for terrain.

use bevy::prelude::*;

use crate::{
    asset_management::terrain::TerrainHandles,
    player_interaction::selection::{CurrentSelection, HoveredTiles},
    simulation::geometry::{MapGeometry, TilePos},
    terrain::Terrain,
};

/// Adds rendering components to every spawned terrain tile
pub(super) fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    handles: Res<TerrainHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, tile_pos, terrain) in new_terrain.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let hex_height = *map_geometry.height_index.get(tile_pos).unwrap();

        commands.entity(terrain_entity).insert(PbrBundle {
            mesh: handles.mesh.clone_weak(),
            material: handles.terrain_materials.get(terrain).unwrap().clone_weak(),
            transform: Transform::from_xyz(pos.x, 0.0, pos.y).with_scale(Vec3 {
                x: 1.,
                y: hex_height,
                z: 1.,
            }),
            ..default()
        });
    }
}

/// Shows which tiles are being hovered and selected.
pub(super) fn display_tile_interactions(
    current_selection: Res<CurrentSelection>,
    hovered_tiles: Res<HoveredTiles>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TerrainHandles>,
) {
    if current_selection.is_changed() || hovered_tiles.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = hovered_tiles.contains(&tile_pos);
            let selected = if let CurrentSelection::Terrain(selected_tiles) = &*current_selection {
                selected_tiles.contains_tile(tile_pos)
            } else {
                false
            };

            *material = materials.get_material(terrain, hovered, selected);
        }
    }
}
