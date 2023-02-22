//! Graphics and animation code for units.

use bevy::prelude::*;

use crate::{
    asset_management::units::UnitHandles,
    simulation::geometry::{MapGeometry, TilePos},
    units::{item_interaction::HeldItem, UnitId},
};

/// Adds rendering components to every spawned unit
pub(super) fn populate_units(
    new_units: Query<(Entity, &TilePos, &UnitId), Added<UnitId>>,
    mut commands: Commands,
    unit_handles: Res<UnitHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (entity, tile_pos, unit_id) in new_units.iter() {
        let scene_handle = unit_handles.scenes.get(unit_id).unwrap();

        commands
            .entity(entity)
            .insert(SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(tile_pos.into_world_pos(&*map_geometry)),
                ..default()
            })
            .insert(unit_handles.picking_mesh.clone_weak());
    }
}

/// Shows the item that each unit is holding
pub(super) fn display_held_item(unit_query: Query<&HeldItem, (With<UnitId>, Changed<HeldItem>)>) {
    for _held_item in unit_query.iter() {
        // TODO: actually display this
    }
}
