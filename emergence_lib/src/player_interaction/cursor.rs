//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem};

use super::InteractionSystem;
use crate::{simulation::geometry::TilePos, terrain::Terrain};

/// Initializes the [`CursorWorldPos`] and [`CursorTilePos`] resources, which are kept updated  
/// updated using [`update_cursor_pos`].
pub struct CursorTilePosPlugin;

impl Plugin for CursorTilePosPlugin {
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
    /// The terrain entity that the cursor is over top of.
    terrain_entity: Option<Entity>,
    /// The tile position that the cursor is over top of.
    tile_pos: Option<TilePos>,
}

impl CursorPos {
    /// The position of the cursor in hex coordinates, if it is on the hex map.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub fn maybe_tile_pos(&self) -> Option<TilePos> {
        self.tile_pos
    }

    /// The terrain entity under the cursor, if any.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub fn maybe_entity(&self) -> Option<Entity> {
        self.terrain_entity
    }
}

/// Updates the raycast with the cursor position
///
/// This system was borrowed from https://github.com/aevyrie/bevy_mod_raycast/blob/79012e4c7b12896ccfed09a129d163726d3a6516/examples/mouse_picking.rs#L45
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
) {
    let raycast_source = camera_query.single();
    let maybe_intersection = raycast_source.get_nearest_intersection();

    if let Some((terrain_entity, _intersection_data)) = maybe_intersection {
        cursor_pos.terrain_entity = Some(terrain_entity);
        cursor_pos.tile_pos = terrain_query.get(terrain_entity).ok().copied();
    } else {
        cursor_pos.terrain_entity = None;
        cursor_pos.tile_pos = None;
    }
}
