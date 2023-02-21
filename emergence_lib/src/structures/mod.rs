//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{prelude::*, utils::HashMap};
use bevy_mod_raycast::RaycastMesh;

use crate::{
    asset_management::manifest::Manifest,
    items::{inventory::Inventory, recipe::RecipeId, ItemCount, ItemId},
    player_interaction::{clipboard::StructureData, selection::ObjectInteraction},
    simulation::geometry::{Facing, TilePos},
};

use self::crafting::{CraftingPlugin, InputInventory};
use std::fmt::Display;

pub(crate) mod commands;
pub(crate) mod crafting;
pub(crate) mod ghost;

/// The data definitions for all structures.
pub(crate) type StructureManifest = Manifest<StructureId, StructureVariety>;

impl StructureManifest {
    /// The color associated with this structure.
    pub(crate) fn color(&self, structure_id: StructureId) -> Color {
        self.get(structure_id).color
    }
}

/// Information about a single [`StructureId`] variety of structure.
#[derive(Debug, Clone)]
pub(crate) struct StructureVariety {
    /// Is this structure alive?
    organism: bool,
    /// Can this structure make things?
    crafts: bool,
    /// Does this structure start with a recipe pre-selected?
    starting_recipe: Option<RecipeId>,
    /// The set of items needed to create a new copy of this structure
    construction_materials: InputInventory,
    /// The color associated with this structure
    color: Color,
}

impl Default for StructureManifest {
    fn default() -> Self {
        let mut map = HashMap::default();

        let leuco_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(ItemCount::new(ItemId::leuco_chunk(), 1)),
        };

        // TODO: read these from files
        map.insert(
            StructureId { id: "leuco" },
            StructureVariety {
                organism: true,
                crafts: true,
                starting_recipe: Some(RecipeId::leuco_chunk_production()),
                construction_materials: leuco_construction_materials,
                color: Color::ORANGE_RED,
            },
        );

        let acacia_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(ItemCount::new(ItemId::acacia_leaf(), 2)),
        };

        map.insert(
            StructureId { id: "acacia" },
            StructureVariety {
                organism: true,
                crafts: true,
                starting_recipe: Some(RecipeId::acacia_leaf_production()),
                construction_materials: acacia_construction_materials,
                color: Color::GREEN,
            },
        );

        StructureManifest::new(map)
    }
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
    /// How is this structure being interacted with
    object_interaction: ObjectInteraction,
}

impl StructureBundle {
    /// Creates a new structure
    fn new(tile_pos: TilePos, data: StructureData) -> Self {
        StructureBundle {
            structure: data.structure_id,
            facing: data.facing,
            tile_pos,
            raycast_mesh: RaycastMesh::default(),
            object_interaction: ObjectInteraction::None,
        }
    }
}

/// Structures are static buildings that take up one or more tile
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct StructureId {
    /// The unique identifier for this variety of structure.
    pub(crate) id: &'static str,
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
        app.add_plugin(CraftingPlugin)
            .init_resource::<StructureManifest>();
    }
}
