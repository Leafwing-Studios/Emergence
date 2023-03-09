//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem};
use leafwing_input_manager::prelude::ActionState;

use super::{InteractionSystem, PlayerAction};
use crate::{
    asset_management::manifest::{Id, Structure, Unit},
    simulation::geometry::TilePos,
    structures::ghost::Ghost,
    terrain::Terrain,
};

/// Controls raycasting and cursor aethetics.
pub(super) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugin(DefaultRaycastingPlugin::<Terrain>::default())
            .add_plugin(DefaultRaycastingPlugin::<Id<Structure>>::default())
            .add_plugin(DefaultRaycastingPlugin::<Id<Unit>>::default())
            .add_plugin(DefaultRaycastingPlugin::<Ghost>::default())
            .add_system(
                update_raycast_with_cursor
                    .before(RaycastSystem::BuildRays::<Terrain>)
                    .in_base_set(CoreSet::First),
            )
            .add_system(move_cursor_manually.in_base_set(CoreSet::PreUpdate))
            .add_system(
                update_cursor_pos
                    .in_set(InteractionSystem::ComputeCursorPos)
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
    /// The first ghost hit by a cursor raycast, if any.
    hovered_ghost: Option<Entity>,
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

    /// The hovered ghost, if available.
    pub(crate) fn maybe_ghost(&self) -> Option<Entity> {
        self.hovered_ghost
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
            &mut RaycastSource<Id<Structure>>,
            &mut RaycastSource<Id<Unit>>,
            &mut RaycastSource<Ghost>,
        ),
        With<Camera>,
    >,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for (mut terrain_raycast, mut structure_raycast, mut unit_raycast, mut ghost_raycast) in
        query.iter_mut()
    {
        terrain_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        structure_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        unit_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        ghost_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

/// Updates the location of the cursor and what it is hovering over
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    camera_query: Query<
        (
            &mut RaycastSource<Terrain>,
            &mut RaycastSource<Id<Structure>>,
            &mut RaycastSource<Id<Unit>>,
            &mut RaycastSource<Ghost>,
        ),
        With<Camera>,
    >,
    terrain_query: Query<&TilePos, With<Terrain>>,
    structure_query: Query<Entity, With<Id<Structure>>>,
    unit_query: Query<Entity, With<Id<Unit>>>,
    ghost_query: Query<Entity, With<Ghost>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    let (terrain_raycast, structure_raycast, unit_raycast, ghost_raycast) = camera_query.single();

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

    cursor_pos.hovered_ghost = if let Some((ghost_entity, _intersection_data)) =
        ghost_raycast.get_nearest_intersection()
    {
        ghost_query.get(ghost_entity).ok()
    } else {
        None
    };

    if let Some(last_mouse_position) = cursor_moved_events.iter().last() {
        cursor_pos.screen_pos = Some(last_mouse_position.position);
    }
}

/// Moves the cursor on the screen, based on gamepad or keyboard inputs
fn move_cursor_manually(
    actions: Res<ActionState<PlayerAction>>,
    mut window_query: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut cursor_moved_events: EventWriter<CursorMoved>,
) {
    /// Controls the sensitivity of cursor movement
    const CURSOR_SPEED: f32 = 2.0;

    if let Ok((primary_window_entity, mut primary_window)) = window_query.get_single_mut() {
        let maybe_cursor_pos = primary_window.cursor_position();

        if let Some(old_cursor_pos) = maybe_cursor_pos {
            if let Some(raw_delta) = actions.axis_pair(PlayerAction::MoveCursor) {
                let delta = raw_delta.xy() * CURSOR_SPEED;

                if delta != Vec2::ZERO {
                    let new_cursor_pos = old_cursor_pos + delta;
                    primary_window.set_cursor_position(Some(new_cursor_pos));
                    cursor_moved_events.send(CursorMoved {
                        window: primary_window_entity,
                        position: new_cursor_pos,
                    });
                }
            }
        }
    }
}
