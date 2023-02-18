//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{prelude::*, reflect::TypeUuid};
use bevy_mod_raycast::RaycastMesh;
use serde::Deserialize;

use crate::{
    asset_management::manifest::Manifest,
    items::recipe::RecipeId,
    player_interaction::clipboard::StructureData,
    simulation::geometry::{Facing, TilePos},
};

use self::crafting::CraftingPlugin;
use std::fmt::Display;

pub(crate) mod commands;
pub(crate) mod crafting;
pub(crate) mod ghost;

/// The data definitions for all structures.
pub(crate) type StructureManifest = Manifest<StructureId, StructureVariety>;

impl StructureManifest {
    /// The color associated with this structure.
    pub(crate) fn color(&self, structure_id: &StructureId) -> Color {
        self.get(structure_id).color
    }
}

/// Information about a single [`StructureId`] variety of structure.
#[derive(Debug, Clone, TypeUuid, Deserialize)]
#[uuid = "a58bfc5a-3289-4351-b7be-ac8fdbe7ff8b"]
pub(crate) struct StructureVariety {
    /// Is this structure alive?
    organism: bool,
    /// Can this structure make things?
    crafts: bool,
    /// Does this structure start with a recipe pre-selected?
    starting_recipe: Option<RecipeId>,
    /// The color associated with this structure
    color: Color,
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Unique identifier of structure variety
    structure: StructureId,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<StructureId>,
}

impl StructureBundle {
    /// Creates a new structure
    fn new(tile_pos: TilePos, data: StructureData) -> Self {
        StructureBundle {
            structure: data.id,
            facing: data.facing,
            tile_pos,
            raycast_mesh: RaycastMesh::default(),
        }
    }
}

/// Structures are static buildings that take up one or more tile
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug, TypeUuid, Deserialize)]
#[uuid = "a161ca1d-e024-4fe9-bedc-f4352d6d99c0"]
pub(crate) struct StructureId {
    /// The unique identifier for this variety of structure.
    pub(crate) id: u32,
}

impl From<u32> for StructureId {
    fn from(value: u32) -> Self {
        Self { id: value }
    }
}

impl Display for StructureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// The systems that make structures tick.
pub(super) struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        // FIXME: Load structure manifest
        app.add_plugin(CraftingPlugin);
    }
}
