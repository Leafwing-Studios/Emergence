//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem};

use super::InteractionSystem;
use crate::{simulation::geometry::TilePos, terrain::Terrain};

/// Controls raycasting and cursor aethetics.
pub struct CursorPlugin;

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

/// The tile position of the mouse cursor, if it lies over the map.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CursorPos {
    /// The tile position that the cursor is over top of.
    tile_pos: Option<TilePos>,
    /// The screen position of the cursor.
    ///
    /// Measured from the top-left corner in logical units.
    screen_pos: Option<Vec2>,
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
}

/// Updates the raycast with the cursor position
///
/// This system was borrowed from <https://github.com/aevyrie/bevy_mod_raycast/blob/79012e4c7b12896ccfed09a129d163726d3a6516/examples/mouse_picking.rs#L45>
/// and used under the MIT License. Thanks!
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<Terrain>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

/// Records which tile is currently under the cursor, if any
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    camera_query: Query<&RaycastSource<Terrain>, With<Camera>>,
    terrain_query: Query<&TilePos, With<Terrain>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    let raycast_source = camera_query.single();
    let maybe_intersection = raycast_source.get_nearest_intersection();

    if let Some((terrain_entity, _intersection_data)) = maybe_intersection {
        cursor_pos.tile_pos = terrain_query.get(terrain_entity).ok().copied();
    } else {
        cursor_pos.tile_pos = None;
    }

    if let Some(last_mouse_position) = cursor_moved_events.iter().last() {
        cursor_pos.screen_pos = Some(last_mouse_position.position);
    }
}
