//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{prelude::*, utils::HashMap};

use crate::{
    items::recipe::RecipeId,
    player_interaction::clipboard::StructureData,
    simulation::geometry::{Facing, TilePos},
};

use self::crafting::CraftingPlugin;

pub(crate) mod commands;
pub(crate) mod crafting;
pub(crate) mod ghost;

/// A central lookup for how each variety the structure works.
#[derive(Resource, Debug, Deref, DerefMut)]
struct StructureInfo {
    /// A simple lookup table
    map: HashMap<StructureId, StructureVariety>,
}

/// Information about a single [`StructureId`] variety of structure.
#[derive(Debug, Clone)]
struct StructureVariety {
    /// Is this structure alive?
    organism: bool,
    /// Can this structure make things?
    crafts: bool,
    /// Does this structure start with a recipe pre-selected?
    starting_recipe: Option<RecipeId>,
}

impl Default for StructureInfo {
    fn default() -> Self {
        let mut map = HashMap::default();

        // TODO: read these from files
        map.insert(
            StructureId::new("leuco"),
            StructureVariety {
                organism: true,
                crafts: true,
                starting_recipe: None,
            },
        );

        map.insert(
            StructureId::new("acacia"),
            StructureVariety {
                organism: true,
                crafts: true,
                starting_recipe: Some(RecipeId::acacia_leaf_production()),
            },
        );

        StructureInfo { map }
    }
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Data characterizing structures
    structure: StructureId,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
}

impl StructureBundle {
    /// Creates a new structure
    pub fn new(tile_pos: TilePos, data: StructureData) -> Self {
        StructureBundle {
            structure: data.id,
            facing: data.facing,
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
        app.add_plugin(CraftingPlugin)
            .init_resource::<StructureInfo>();
    }
}
