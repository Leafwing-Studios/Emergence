//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.
//!
//! There is an important distinction between "ghosts" and "previews", even though they appear similarly to players.
//! Ghosts are buildings that are genuinely planned to be built.
//! Previews are simply hovered, and used as a visual aid to show placement.

use bevy::prelude::*;

use crate::{
    player_interaction::clipboard::StructureData,
    simulation::geometry::{Facing, TilePos},
};

use super::{crafting::InputInventory, StructureId};

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// A marker component indicating that this structure should be rendered in a transparent style.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghostly;

/// The set of components needed to spawn a ghost.
#[derive(Bundle)]
pub(super) struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// Render this entity in a translucent style
    ghostly: Ghostly,
    /// The location of the ghost
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: StructureId,
    /// The direction the ghost is facing
    facing: Facing,
    /// The items required to actually seed this item
    construction_materials: InputInventory,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    pub(super) fn new(
        tile_pos: TilePos,
        data: StructureData,
        construction_materials: InputInventory,
    ) -> Self {
        GhostBundle {
            ghost: Ghost,
            ghostly: Ghostly,
            tile_pos,
            structure_id: data.structure_id,
            facing: data.facing,
            construction_materials,
        }
    }
}

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Preview;

/// The set of components needed to spawn a structure preview.
#[derive(Bundle)]
pub(super) struct PreviewBundle {
    /// Marker component
    preview: Preview,
    /// Render this entity in a translucent style
    ghostly: Ghostly,
    /// The location of the preview
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: StructureId,
    /// The direction the preview is facing
    facing: Facing,
}

impl PreviewBundle {
    /// Creates a new [`PreviewBundle`].
    pub(super) fn new(tile_pos: TilePos, data: StructureData) -> Self {
        PreviewBundle {
            preview: Preview,
            ghostly: Ghostly,
            tile_pos,
            structure_id: data.structure_id,
            facing: data.facing,
        }
    }
}
