//! Keep track of the mouse cursor in world space, and convert it into a tile position, if
//! available.
use bevy::prelude::*;
use bevy_mod_picking::{DebugEventsPickingPlugin, InteractablePickingPlugin, PickingPlugin};
use hexx::Hex;

use crate::simulation::geometry::TilePos;

use super::InteractionSystem;

/// Initializes the [`CursorWorldPos`] and [`CursorTilePos`] resources, which are kept updated  
/// updated using [`update_cursor_pos`].
pub struct CursorTilePosPlugin;

impl Plugin for CursorTilePosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(DebugEventsPickingPlugin)
            .add_system(
                update_cursor_pos
                    .label(InteractionSystem::ComputeCursorPos)
                    .after(InteractionSystem::MoveCamera),
            );
    }
}

/// The tile position of the mouse cursor, if it lies over the map.
#[derive(Resource, Default, Clone, Copy)]
pub struct CursorPos(Option<TilePos>);

impl CursorPos {
    /// The position of the cursor in hex coordinates, if it is on the hex map.
    ///
    /// If the cursor is outside the map, this will return `None`.
    pub fn maybe_tile_pos(&self) -> Option<TilePos> {
        self.0
    }
}

/// Updates which tile the cursor is hovering over
pub fn update_cursor_pos(mut cursor_pos: ResMut<CursorPos>) {
    cursor_pos.0 = Some(TilePos { hex: Hex::ZERO });
}
