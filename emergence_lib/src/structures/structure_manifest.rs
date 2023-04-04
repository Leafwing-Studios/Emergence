//! Defines write-only data for each variety of structure.

use crate::{
    asset_management::manifest::{loader::IsRawManifest, Id, Manifest},
    crafting::components::{ActiveRecipe, InputInventory, RawActiveRecipe},
    items::{item_manifest::Item, slot::ItemSlot},
    organisms::{OrganismId, OrganismVariety, RawOrganismVariety},
    structures::construction::Footprint,
    terrain::terrain_manifest::Terrain,
};
use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::{Duration, HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

/// The marker type for [`Id<Structure>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Structure;
/// Stores the read-only definitions for all structures.
pub type StructureManifest = Manifest<Structure, StructureData>;

impl StructureManifest {
    /// Fetches the [`ConstructionData`] for a given structure type.
    pub fn construction_data(&self, structure_id: Id<Structure>) -> &ConstructionData {
        let initial_strategy = &self.get(structure_id).construction_strategy;
        match initial_strategy {
            ConstructionStrategy::Seedling(seedling_id) => self.construction_data(*seedling_id),
            ConstructionStrategy::Direct(data) => data,
        }
    }
}

/// Information about a single [`Id<Structure>`] variety of structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureData {
    /// Data needed for living structures
    pub organism_variety: Option<OrganismVariety>,
    /// What base variety of structure is this?
    ///
    /// Determines the components that this structure gets.
    pub kind: StructureKind,
    /// How new copies of this structure can be built
    pub construction_strategy: ConstructionStrategy,
    /// The maximum number of workers that can work at this structure at once.
    pub max_workers: u8,
    /// The tiles taken up by this building.
    pub footprint: Footprint,
}

/// The unprocessed equivalent of [`StructureData`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawStructureData {
    /// Data needed for living structures
    pub organism_variety: Option<RawOrganismVariety>,
    /// What base variety of structure is this?
    ///
    /// Determines the components that this structure gets.
    pub kind: RawStructureKind,
    /// How new copies of this structure can be built
    pub construction_strategy: RawConstructionStrategy,
    /// The maximum number of workers that can work at this structure at once.
    pub max_workers: u8,
    /// The tiles taken up by this building.
    pub footprint: Option<Footprint>,
}

impl From<RawStructureData> for StructureData {
    fn from(raw: RawStructureData) -> Self {
        Self {
            organism_variety: raw.organism_variety.map(Into::into),
            kind: raw.kind.into(),
            construction_strategy: raw.construction_strategy.into(),
            max_workers: raw.max_workers,
            footprint: raw.footprint.unwrap_or_default(),
        }
    }
}

/// How new structures of this sort can be built.
///
/// For structures that are part of a `Lifecycle`, this should generally be the same for all of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstructionStrategy {
    /// Follows the construction strategy of another structure.
    Seedling(Id<Structure>),
    /// This structure can be built directly.
    Direct(ConstructionData),
}

/// The data contained in a [`ConstructionStrategy::Direct`] variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionData {
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`None`], no work will be needed at all.
    pub work: Option<Duration>,
    /// The set of items needed to create a new copy of this structure
    pub materials: InputInventory,
    /// The set of terrain types that this structure can be built on
    pub allowed_terrain_types: HashSet<Id<Terrain>>,
}

/// The unprocessed equivalent of [`ConstructionStrategy`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RawConstructionStrategy {
    /// Follows the construction strategy of another structure.
    Seedling(String),
    /// This structure can be built directly.
    Direct {
        /// The amount of work (in seconds) by units required to complete the construction of this building.
        ///
        /// If this is [`None`], no work will be needed at all.
        work: Option<f32>,
        /// The set of items needed to create a new copy of this structure
        materials: HashMap<String, usize>,
        /// The set of terrain types that this structure can be built on
        allowed_terrain_types: HashSet<String>,
    },
}

impl From<RawConstructionStrategy> for ConstructionStrategy {
    fn from(raw: RawConstructionStrategy) -> Self {
        match raw {
            RawConstructionStrategy::Seedling(seedling_name) => {
                ConstructionStrategy::Seedling(Id::from_name(seedling_name))
            }
            RawConstructionStrategy::Direct {
                work,
                materials,
                allowed_terrain_types,
            } => {
                let inventory = materials
                    .into_iter()
                    .map(|(item_name, count)| ItemSlot::new(Id::from_name(item_name), count))
                    .collect();

                let materials = InputInventory { inventory };
                ConstructionStrategy::Direct(ConstructionData {
                    work: work.map(Duration::from_secs_f32),
                    materials,
                    allowed_terrain_types: allowed_terrain_types
                        .into_iter()
                        .map(Id::from_name)
                        .collect(),
                })
            }
        }
    }
}

/// What set of components should this structure have?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureKind {
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

/// The unprocessed equivalent of [`StructureKind`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawStructureKind {
    /// Stores items.
    Storage {
        /// The number of slots in the inventory, controlling how large it is.
        max_slot_count: usize,
        /// Is any item allowed here, or just one?
        reserved_for: Option<String>,
    },
    /// Crafts items, turning inputs into outputs.
    Crafting {
        /// Does this structure start with a recipe pre-selected?
        starting_recipe: RawActiveRecipe,
    },
}

impl From<RawStructureKind> for StructureKind {
    fn from(raw: RawStructureKind) -> Self {
        match raw {
            RawStructureKind::Storage {
                max_slot_count,
                reserved_for,
            } => Self::Storage {
                max_slot_count,
                reserved_for: reserved_for.map(Id::from_name),
            },
            RawStructureKind::Crafting { starting_recipe } => Self::Crafting {
                starting_recipe: starting_recipe.into(),
            },
        }
    }
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

/// The [`StructureManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "77ddfe49-be99-4fea-bbba-0c085821f6b8"]
pub struct RawStructureManifest {
    /// The data for each structure.
    pub structure_types: HashMap<String, RawStructureData>,
}

impl IsRawManifest for RawStructureManifest {
    const EXTENSION: &'static str = "structure_manifest.json";

    type Marker = Structure;
    type Data = StructureData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.structure_types.clone() {
            let data = raw_data.into();

            manifest.insert(raw_id, data)
        }

        manifest
    }
}
