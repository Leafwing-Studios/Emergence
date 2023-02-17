//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem};

use super::InteractionSystem;
use crate::{
    organisms::units::UnitId, simulation::geometry::TilePos, structures::StructureId,
    terrain::Terrain,
};

/// Controls raycasting and cursor aethetics.
pub(super) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugin(DefaultRaycastingPlugin::<Terrain>::default())
            .add_system_to_stage(
                CoreStage::First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<Terrain>),
            )
            .add_system(
                update_cursor_pos
                    .label(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::MoveCamera),
            );
    }
}

/// The position of the mouse cursor and what it is hovering over.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub(crate) struct CursorPos {
    /// The tile position that the cursor is over top of.
    tile_pos: Option<TilePos>,
    /// The screen position of the cursor.
    ///
    /// Measured from the top-left corner in logical units.
    screen_pos: Option<Vec2>,
    /// The first unit hit by a cursor raycast, if any.
    hovered_unit: Option<Entity>,
    /// The first structure hit by a cursor raycast, if any.
    hovered_structure: Option<Entity>,
}

impl CursorPos {
    /// The position of the cursor in hex coordinates, if it is on the hex map.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub(crate) fn maybe_tile_pos(&self) -> Option<TilePos> {
        self.tile_pos
    }

    /// The position of the cursor on the screen, if available.
    pub(crate) fn maybe_screen_pos(&self) -> Option<Vec2> {
        self.screen_pos
    }

    /// The hovered unit, if available.
    pub(crate) fn maybe_unit(&self) -> Option<Entity> {
        self.hovered_unit
    }

    /// The hovered structure, if available.
    pub(crate) fn maybe_structure(&self) -> Option<Entity> {
        self.hovered_structure
    }
}

/// Updates the raycast with the cursor position
///
/// This system was adapted from <https://github.com/aevyrie/bevy_mod_raycast/blob/79012e4c7b12896ccfed09a129d163726d3a6516/examples/mouse_picking.rs#L45>
/// and used under the MIT License. Thanks!
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<
        (
            &mut RaycastSource<Terrain>,
            &mut RaycastSource<StructureId>,
            &mut RaycastSource<UnitId>,
        ),
        With<Camera>,
    >,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for (mut terrain_raycast, mut structure_raycast, mut unit_raycast) in query.iter_mut() {
        terrain_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        structure_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        unit_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

/// Records which tile is currently under the cursor, if any
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    camera_query: Query<
        (
            &mut RaycastSource<Terrain>,
            &mut RaycastSource<StructureId>,
            &mut RaycastSource<UnitId>,
        ),
        With<Camera>,
    >,
    terrain_query: Query<&TilePos, With<Terrain>>,
    structure_query: Query<Entity, With<StructureId>>,
    unit_query: Query<Entity, With<UnitId>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    let (terrain_raycast, structure_raycast, unit_raycast) = camera_query.single();

    cursor_pos.tile_pos = if let Some((terrain_entity, _intersection_data)) =
        terrain_raycast.get_nearest_intersection()
    {
        terrain_query.get(terrain_entity).ok().copied()
    } else {
        None
    };

    cursor_pos.hovered_structure = if let Some((structure_entity, _intersection_data)) =
        structure_raycast.get_nearest_intersection()
    {
        structure_query.get(structure_entity).ok()
    } else {
        None
    };

    cursor_pos.hovered_unit =
        if let Some((unit_entity, _intersection_data)) = unit_raycast.get_nearest_intersection() {
            unit_query.get(unit_entity).ok()
        } else {
            None
        };

    if let Some(last_mouse_position) = cursor_moved_events.iter().last() {
        cursor_pos.screen_pos = Some(last_mouse_position.position);
    }
}
