//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.

use bevy::prelude::*;

use crate::{
    player_interaction::clipboard::ClipboardItem,
    simulation::geometry::{Facing, TilePos},
};

use super::StructureId;

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// The set of components needed to spawn a ghost.
#[derive(Bundle)]
pub(super) struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// The location of the ghost
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: StructureId,
    /// The direction the ghost is facing
    facing: Facing,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    pub fn new(tile_pos: TilePos, item: ClipboardItem) -> Self {
        GhostBundle {
            ghost: Ghost,
            tile_pos,
            structure_id: item.id,
            facing: item.facing,
        }
    }
}
