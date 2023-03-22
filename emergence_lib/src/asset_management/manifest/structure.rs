//! Data and manifest definitions for structure.

use crate::{
    items::inventory::Inventory,
    organisms::{
        energy::{Energy, EnergyPool},
        OrganismVariety,
    },
    structures::crafting::{ActiveRecipe, InputInventory},
};
use bevy::utils::{Duration, HashSet};

use leafwing_abilities::prelude::Pool;

use super::{Id, Item, Manifest, StructureManifest, Terrain};

/// Information about a single [`Id<Structure>`] variety of structure.
#[derive(Debug, Clone)]
pub(crate) struct StructureData {
    /// Is this the "base" form that we should display to players in menus and for ghosts?
    pub(crate) prototypical: bool,
    /// Data needed for living structures
    pub(crate) organism: Option<OrganismVariety>,
    /// What base variety of structure is this?
    ///
    /// Determines the components that this structure gets.
    pub(crate) kind: StructureKind,
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`Duration::ZERO`], no work will be needed at all.
    pub(crate) build_duration: Duration,
    /// The set of items needed to create a new copy of this structure
    pub(crate) construction_materials: InputInventory,
    /// The set of terrain types that this structure can be built on
    pub(crate) allowed_terrain_types: HashSet<Id<Terrain>>,
}

/// What set of components should this structure have?
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StructureKind {
    /// Stores items.
    Storage {
        /// The number of slots in the inventory, controlling how large it is.
        max_slot_count: usize,
        /// Is any item allowed here, or just one?
        reserved_for: Option<Id<Item>>,
    },
    /// Crafts items, turning inputs into outputs.
    Crafting {
        /// Does this structure start with a recipe pre-selected?
        starting_recipe: ActiveRecipe,
    },
}

impl StructureData {
    /// Returns the starting recipe of the structure
    ///
    /// If no starting recipe is set, [`ActiveRecipe::NONE`] will be returned.
    pub fn starting_recipe(&self) -> &ActiveRecipe {
        if let StructureKind::Crafting { starting_recipe } = &self.kind {
            starting_recipe
        } else {
            &ActiveRecipe::NONE
        }
    }

    /// Returns the set of terrain types that this structure can be built on
    pub fn allowed_terrain_types(&self) -> &HashSet<Id<Terrain>> {
        &self.allowed_terrain_types
    }
}

impl Default for StructureManifest {
    fn default() -> Self {
        let mut manifest: StructureManifest = Manifest::new();

        let leuco_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(Id::from_name("leuco_chunk"), 1),
        };

        // TODO: read these from files
        manifest.insert(
            "leuco",
            StructureData {
                prototypical: true,
                organism: Some(OrganismVariety {
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("leuco_chunk_production")),
                },
                build_duration: Duration::from_secs(5),
                construction_materials: leuco_construction_materials,
                allowed_terrain_types: HashSet::from_iter([
                    Id::from_name("loam"),
                    Id::from_name("muddy"),
                ]),
            },
        );

        let acacia_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(Id::from_name("acacia_leaf"), 2),
        };

        manifest.insert(
            "acacia",
            StructureData {
                prototypical: true,
                organism: Some(OrganismVariety {
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("acacia_leaf_production")),
                },
                build_duration: Duration::ZERO,
                construction_materials: acacia_construction_materials,
                allowed_terrain_types: HashSet::from_iter([
                    Id::from_name("loam"),
                    Id::from_name("muddy"),
                ]),
            },
        );

        manifest.insert(
            "ant_hive",
            StructureData {
                prototypical: true,
                organism: None,
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("ant_egg_production")),
                },
                construction_materials: InputInventory::default(),
                build_duration: Duration::from_secs(10),
                allowed_terrain_types: HashSet::from_iter([
                    Id::from_name("loam"),
                    Id::from_name("muddy"),
                    Id::from_name("rocky"),
                ]),
            },
        );

        manifest.insert(
            "hatchery",
            StructureData {
                prototypical: true,
                organism: None,
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("hatch_ants")),
                },
                construction_materials: InputInventory::default(),
                build_duration: Duration::from_secs(5),
                allowed_terrain_types: HashSet::from_iter([
                    Id::from_name("loam"),
                    Id::from_name("muddy"),
                    Id::from_name("rocky"),
                ]),
            },
        );

        manifest.insert(
            "storage",
            StructureData {
                prototypical: true,
                organism: None,
                kind: StructureKind::Storage {
                    max_slot_count: 3,
                    reserved_for: None,
                },
                construction_materials: InputInventory::default(),
                build_duration: Duration::from_secs(10),
                allowed_terrain_types: HashSet::from_iter([
                    Id::from_name("loam"),
                    Id::from_name("muddy"),
                    Id::from_name("rocky"),
                ]),
            },
        );

        manifest
    }
}
