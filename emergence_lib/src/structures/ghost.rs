//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.

use bevy::prelude::*;

use crate::simulation::geometry::TilePos;

use super::StructureId;

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// The set of components needed to spawn a ghost.
#[derive(Bundle)]
pub(super) struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// The variety of structure
    structure_id: StructureId,
    /// The location of the ghost
    tile_pos: TilePos,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    pub(super) fn new(id: StructureId, tile_pos: TilePos) -> Self {
        GhostBundle {
            ghost: Ghost,
            structure_id: id,
            tile_pos,
        }
    }
}
