//! Defines write-only data for each variety of structure.

use crate::{
    asset_management::manifest::{loader::IsRawManifest, Id, Manifest, RawId},
    items::item_manifest::Item,
    organisms::{OrganismId, OrganismVariety, RawOrganismVariety},
    structures::{
        construction::Footprint,
        crafting::{ActiveRecipe, InputInventory},
    },
    terrain::terrain_manifest::Terrain,
};
use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::{Duration, HashMap, HashSet},
};

use serde::{Deserialize, Serialize};

use super::crafting::RawActiveRecipe;

/// The marker type for [`Id<Structure>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Structure;
/// Stores the read-only definitions for all structures.
pub type StructureManifest = Manifest<Structure, StructureData>;

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
    pub construction_strategy: ConstructionStrategy,
    /// The maximum number of workers that can work at this structure at once.
    pub max_workers: u8,
    /// The tiles taken up by this building.
    pub footprint: Footprint,
}

impl From<RawStructureData> for StructureData {
    fn from(raw: RawStructureData) -> Self {
        Self {
            organism_variety: raw.organism_variety.map(Into::into),
            kind: raw.kind.into(),
            construction_strategy: raw.construction_strategy,
            max_workers: raw.max_workers,
            footprint: raw.footprint,
        }
    }
}

/// How new structures of this sort can be built.
///
/// For structures that are part of a `Lifecycle`, this should generally be the same for all of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionStrategy {
    /// The "seedling" or "baby" form of this structure that should be built when we attempt to build a structure of this type.
    ///
    /// If `None`, this structure can be built directly.
    pub seedling: Option<Id<Structure>>,
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`Duration::ZERO`], no work will be needed at all.
    pub work: Duration,
    /// The set of items needed to create a new copy of this structure
    pub materials: InputInventory,
    /// The set of terrain types that this structure can be built on
    pub allowed_terrain_types: HashSet<Id<Terrain>>,
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
        ///
        /// A value of [`None`] is represented by the empty string.
        reserved_for: String,
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
                reserved_for: if reserved_for.is_empty() {
                    None
                } else {
                    Some(Id::from_name(&reserved_for))
                },
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

/// The [`StructureManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "77ddfe49-be99-4fea-bbba-0c085821f6b8"]
pub struct RawStructureManifest {
    /// The data for each structure.
    pub structure_types: HashMap<RawId<Structure>, RawStructureData>,
}

impl IsRawManifest for RawStructureManifest {
    const EXTENSION: &'static str = "structure_manifest.json";

    type Marker = Structure;
    type Data = StructureData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.structure_types.clone() {
            let data = raw_data.into();

            manifest.insert(raw_id.name(), data)
        }

        manifest
    }
}
