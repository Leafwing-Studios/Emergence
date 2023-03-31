//! Data and manifest definitions for structure.

use crate::{
    asset_management::manifest::{Id, Item, Manifest, Terrain},
    items::inventory::Inventory,
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::{LifePath, Lifecycle},
        OrganismId, OrganismVariety,
    },
    simulation::time::TimePool,
    structures::{
        construction::Footprint,
        crafting::{ActiveRecipe, InputInventory},
    },
};
use bevy::{
    reflect::{FromReflect, Reflect},
    utils::{Duration, HashSet},
};

use leafwing_abilities::prelude::Pool;

/// The marker type for [`Id<Structure>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Structure;
/// Stores the read-only definitions for all structures.
pub(crate) type StructureManifest = Manifest<Structure, StructureData>;

/// Information about a single [`Id<Structure>`] variety of structure.
#[derive(Debug, Clone)]
pub(crate) struct StructureData {
    /// Data needed for living structures
    pub(crate) organism_variety: Option<OrganismVariety>,
    /// What base variety of structure is this?
    ///
    /// Determines the components that this structure gets.
    pub(crate) kind: StructureKind,
    /// How new copies of this structure can be built
    pub(crate) construction_strategy: ConstructionStrategy,
    /// The maximum number of workers that can work at this structure at once.
    pub(crate) max_workers: u8,
    /// The tiles taken up by this building.
    pub(crate) footprint: Footprint,
}

/// How new structures of this sort can be built.
///
/// For structures that are part of a `Lifecycle`, this should generally be the same for all of them.
#[derive(Debug, Clone)]
pub(crate) struct ConstructionStrategy {
    /// The "seedling" or "baby" form of this structure that should be built when we attempt to build a structure of this type.
    ///
    /// If `None`, this structure can be built directly.
    pub(crate) seedling: Option<Id<Structure>>,
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`Duration::ZERO`], no work will be needed at all.
    pub(crate) work: Duration,
    /// The set of items needed to create a new copy of this structure
    pub(crate) materials: InputInventory,
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
        &self.construction_strategy.allowed_terrain_types
    }
}

impl StructureManifest {
    /// Returns the list of [`Id<Structure>`] where [`StructureData`]'s `prototypical` field is `true`.
    ///
    /// These should be used to populate menus and other player-facing tools.
    pub(crate) fn prototypes(&self) -> impl IntoIterator<Item = Id<Structure>> + '_ {
        self.data_map()
            .iter()
            .filter(|(id, v)| match &v.organism_variety {
                None => true,
                Some(variety) => variety.prototypical_form == OrganismId::Structure(**id),
            })
            .map(|(id, _v)| *id)
    }

    /// Returns the names of all structures where [`StructureData`]'s `prototypical` field is `true`.
    ///
    /// These should be used to populate menus and other player-facing tools.
    pub(crate) fn prototype_names(&self) -> impl IntoIterator<Item = &str> {
        let prototypes = self.prototypes();
        prototypes.into_iter().map(|id| self.name(id))
    }
}

impl Default for StructureManifest {
    fn default() -> Self {
        let mut manifest: StructureManifest = Manifest::new();

        // TODO: read these from files
        manifest.insert(
            "leuco",
            StructureData {
                organism_variety: Some(OrganismVariety {
                    prototypical_form: OrganismId::Structure(Id::from_name("leuco")),
                    lifecycle: Lifecycle::STATIC,
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("leuco_chunk_production")),
                },
                construction_strategy: ConstructionStrategy {
                    seedling: None,
                    work: Duration::from_secs(3),
                    materials: InputInventory {
                        inventory: Inventory::new_from_item(Id::from_name("leuco_chunk"), 1),
                    },
                    allowed_terrain_types: HashSet::from_iter([
                        Id::from_name("loam"),
                        Id::from_name("muddy"),
                    ]),
                },
                max_workers: 6,
                footprint: Footprint::single(),
            },
        );

        let acacia_construction_strategy = ConstructionStrategy {
            seedling: Some(Id::from_name("acacia_seed")),
            work: Duration::ZERO,
            materials: InputInventory {
                inventory: Inventory::new_from_item(Id::from_name("acacia_leaf"), 1),
            },
            allowed_terrain_types: HashSet::from_iter([
                Id::from_name("loam"),
                Id::from_name("muddy"),
            ]),
        };

        manifest.insert(
            "acacia_seed",
            StructureData {
                organism_variety: Some(OrganismVariety {
                    prototypical_form: OrganismId::Structure(Id::from_name("acacia")),
                    lifecycle: Lifecycle::new(vec![LifePath {
                        new_form: OrganismId::Structure(Id::from_name("acacia_sprout")),
                        energy_required: None,
                        time_required: Some(TimePool::simple(1.)),
                    }]),
                    energy_pool: EnergyPool::new_full(Energy(50.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("acacia_leaf_production")),
                },
                construction_strategy: acacia_construction_strategy.clone(),
                max_workers: 1,
                footprint: Footprint::single(),
            },
        );

        manifest.insert(
            "acacia_sprout",
            StructureData {
                organism_variety: Some(OrganismVariety {
                    prototypical_form: OrganismId::Structure(Id::from_name("acacia")),
                    lifecycle: Lifecycle::new(vec![LifePath {
                        new_form: OrganismId::Structure(Id::from_name("acacia")),
                        energy_required: Some(EnergyPool::simple(500.)),
                        time_required: None,
                    }]),
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("acacia_leaf_production")),
                },
                construction_strategy: acacia_construction_strategy.clone(),
                max_workers: 1,
                footprint: Footprint::single(),
            },
        );

        manifest.insert(
            "acacia",
            StructureData {
                organism_variety: Some(OrganismVariety {
                    prototypical_form: OrganismId::Structure(Id::from_name("acacia")),
                    lifecycle: Lifecycle::STATIC,
                    energy_pool: EnergyPool::new_full(Energy(300.), Energy(-1.)),
                }),
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("acacia_leaf_production")),
                },
                construction_strategy: acacia_construction_strategy,
                max_workers: 6,
                footprint: Footprint::single(),
            },
        );

        manifest.insert(
            "ant_hive",
            StructureData {
                organism_variety: None,
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("ant_egg_production")),
                },
                construction_strategy: ConstructionStrategy {
                    seedling: None,
                    work: Duration::from_secs(10),
                    materials: InputInventory::default(),
                    allowed_terrain_types: HashSet::from_iter([
                        Id::from_name("loam"),
                        Id::from_name("muddy"),
                        Id::from_name("rocky"),
                    ]),
                },
                max_workers: 3,
                footprint: Footprint::hexagon(1),
            },
        );

        manifest.insert(
            "hatchery",
            StructureData {
                organism_variety: None,
                kind: StructureKind::Crafting {
                    starting_recipe: ActiveRecipe::new(Id::from_name("hatch_ants")),
                },
                construction_strategy: ConstructionStrategy {
                    seedling: None,
                    work: Duration::from_secs(5),
                    materials: InputInventory::default(),
                    allowed_terrain_types: HashSet::from_iter([
                        Id::from_name("loam"),
                        Id::from_name("muddy"),
                        Id::from_name("rocky"),
                    ]),
                },
                max_workers: 6,
                // Forms a crescent shape
                footprint: Footprint::single(),
            },
        );

        manifest.insert(
            "storage",
            StructureData {
                organism_variety: None,
                kind: StructureKind::Storage {
                    max_slot_count: 3,
                    reserved_for: None,
                },
                construction_strategy: ConstructionStrategy {
                    seedling: None,
                    work: Duration::from_secs(10),
                    materials: InputInventory {
                        inventory: Inventory::new_from_item(Id::from_name("leuco_chunk"), 1),
                    },
                    allowed_terrain_types: HashSet::from_iter([
                        Id::from_name("loam"),
                        Id::from_name("muddy"),
                        Id::from_name("rocky"),
                    ]),
                },
                max_workers: 6,
                footprint: Footprint::single(),
            },
        );

        manifest
    }
}
