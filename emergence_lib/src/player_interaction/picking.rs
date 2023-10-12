//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem};
use leafwing_input_manager::prelude::ActionState;

use super::{InteractionSystem, PlayerAction};
use crate::{asset_management::manifest::Id, geometry::VoxelPos, units::unit_manifest::Unit};

/// Controls raycasting.
pub(super) struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugin(DefaultRaycastingPlugin::<PickableVoxel>::default())
            .add_plugin(DefaultRaycastingPlugin::<Unit>::default())
            .add_systems(
                First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<PickableVoxel>),
            )
            .add_systems(PreUpdate, move_cursor_manually)
            .add_systems(
                Update,
                update_cursor_pos
                    .in_set(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::MoveCamera),
            );
    }
}

/// The position of the mouse cursor and what it is hovering over.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub(crate) struct CursorPos {
    /// The voxel that the cursor is pointing at, if any.
    voxel_pos: Option<VoxelPos>,
    /// The screen position of the cursor.
    ///
    /// Measured from the top-left corner in logical units.
    screen_pos: Option<Vec2>,
    /// The first unit hit by a cursor raycast, if any.
    hovered_unit: Option<Entity>,
}

impl CursorPos {
    /// Creates a new [`CursorPos`] with the given tile position.
    #[cfg(test)]
    pub(crate) fn new(voxel_pos: VoxelPos) -> Self {
        Self {
            voxel_pos: Some(voxel_pos),
            ..Default::default()
        }
    }

    /// The position of the cursor in hex coordinates, if it is on the hex map.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub(crate) fn maybe_voxel_pos(&self) -> Option<VoxelPos> {
        self.voxel_pos
    }

    /// The position of the cursor on the screen, if available.
    pub(crate) fn maybe_screen_pos(&self) -> Option<Vec2> {
        self.screen_pos
    }

    /// The hovered unit, if available.
    pub(crate) fn maybe_unit(&self) -> Option<Entity> {
        self.hovered_unit
    }
}

/// Updates the raycast with the cursor position
///
/// This system was adapted from <https://github.com/aevyrie/bevy_mod_raycast/blob/79012e4c7b12896ccfed09a129d163726d3a6516/examples/mouse_picking.rs#L45>
/// and used under the MIT License. Thanks!
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<(&mut RaycastSource<Unit>, &mut RaycastSource<PickableVoxel>), With<Camera>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for (mut unit_raycast, mut voxel_raycast) in query.iter_mut() {
        unit_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
        voxel_raycast.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

/// A marker that's used to identify meshes and sources for raycasting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(crate) struct PickableVoxel;

/// Updates the location of the cursor and what it is hovering over
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    camera_query: Query<
        (&mut RaycastSource<PickableVoxel>, &mut RaycastSource<Unit>),
        With<Camera>,
    >,
    voxel_query: Query<&VoxelPos>,
    unit_query: Query<Entity, With<Id<Unit>>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    let Ok((voxel_raycast, unit_raycast)) = camera_query.get_single() else {
        return;
    };

    cursor_pos.voxel_pos =
        if let Some((entity, _intersection_data)) = voxel_raycast.get_nearest_intersection() {
            voxel_query.get(entity).ok().copied()
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
