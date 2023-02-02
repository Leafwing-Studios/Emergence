//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;

use crate::simulation::geometry::TilePos;

use super::InteractionSystem;

/// Initializes the [`CursorWorldPos`] and [`CursorTilePos`] resources, which are kept updated  
/// updated using [`update_cursor_pos`].
pub struct CursorTilePosPlugin;

impl Plugin for CursorTilePosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>()
            .init_resource::<CursorTilePos>()
            .add_system(
                update_cursor_pos
                    .label(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::MoveCamera),
            );
    }
}

/// Converts cursor screen position into a world position, taking into account any transforms
/// applied to the camera.
pub fn cursor_pos_in_world(
    windows: &Windows,
    cursor_pos: Vec2,
    cam_t: &Transform,
    cam: &Camera,
) -> Vec3 {
    let window = windows.primary();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}

/// The world position of the mouse cursor.
#[derive(Resource, Clone, Copy, Deref, DerefMut)]
pub struct CursorWorldPos(Vec3);

impl Default for CursorWorldPos {
    fn default() -> Self {
        Self(Vec3::new(f32::INFINITY, f32::INFINITY, 0.0))
    }
}

/// The tile position of the mouse cursor, if it lies over the map.
#[derive(Resource, Default, Clone, Copy)]
pub struct CursorTilePos(Option<TilePos>);

impl CursorTilePos {
    /// The position of the cursor in hex coordinates, if it is on the hex map.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub fn maybe_tile_pos(&self) -> Option<TilePos> {
        self.0
    }
}

/// Updates which tile the cursor is hovering over
pub(super) fn update_cursor_pos() {
    // FIXME: rewrite
}

/// Highlights the current set of selected tiles
pub(super) fn highlight_selected_tiles() {
    // FIXME: rewrite
}
