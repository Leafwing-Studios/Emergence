//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::prelude::*;

use crate::simulation::geometry::{Facing, TilePos};

use self::crafting::CraftingPlugin;

pub mod crafting;

/// The data needed to build a structure
#[derive(Bundle)]
pub struct StructureBundle {
    /// Data characterizing structures
    structure: StructureId,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
}

impl StructureBundle {
    /// Creates a new structure
    pub fn new(id: StructureId, tile_pos: TilePos) -> Self {
        StructureBundle {
            structure: id,
            facing: Facing::default(),
            tile_pos,
        }
    }
}

/// Structures are static buildings that take up one or more tile
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StructureId {
    /// The unique identifier for this variety of structure.
    pub(crate) id: String,
}

impl StructureId {
    /// The size of a single structure
    pub const SIZE: f32 = 1.0;
    /// The offset required to have a structure sit on top of the tile correctly
    pub const OFFSET: f32 = Self::SIZE / 2.0;

    /// Initialize a structure ID via a string.
    pub(crate) fn new(id: &'static str) -> Self {
        StructureId { id: id.to_string() }
    }
}

/// The systems that make structures tick.
pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CraftingPlugin);
    }
}
