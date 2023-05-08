//! Defines write-only data for each variety of structure.

use crate::{
    asset_management::manifest::{loader::IsRawManifest, Id, Manifest},
    construction::{ConstructionData, ConstructionStrategy, RawConstructionStrategy},
    crafting::recipe::{ActiveRecipe, RawActiveRecipe},
    items::item_manifest::Item,
    organisms::{OrganismId, OrganismVariety, RawOrganismVariety},
    simulation::geometry::Height,
    water::roots::RootZone,
};
use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use super::Footprint;

/// The marker type for [`Id<Structure>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Structure;
/// Stores the read-only definitions for all structures.
pub type StructureManifest = Manifest<Structure, StructureData>;

impl StructureManifest {
    /// Fetches the [`ConstructionData`] for a given structure type.
    ///
    /// If the structure uses a seedling, this will recursively fetch the data for the seedling.
    /// If the structure uses a landmark, this will return [`None`].
    pub fn construction_data(&self, structure_id: Id<Structure>) -> Option<&ConstructionData> {
        let initial_strategy = &self.get(structure_id).construction_strategy;
        match initial_strategy {
            ConstructionStrategy::Seedling(seedling_id) => self.construction_data(*seedling_id),
            ConstructionStrategy::Direct(data) => Some(data),
            ConstructionStrategy::Landmark { .. } => None,
        }
    }

    /// Fetches the [`Footprint`] for the initial form of a given structure type.
    pub fn construction_footprint(&self, structure_id: Id<Structure>) -> &Footprint {
        let strategy = &self.get(structure_id).construction_strategy;
        match strategy {
            ConstructionStrategy::Seedling(seedling_id) => {
                self.construction_footprint(*seedling_id)
            }
            ConstructionStrategy::Direct(..) | ConstructionStrategy::Landmark => {
                &self.get(structure_id).footprint
            }
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
    /// The height of the structure, which controls the shadows it casts.
    pub height: Height,
    /// The tiles taken up by this building.
    pub footprint: Footprint,
    /// The set of tiles that this structure can reach with its roots.
    pub root_zone: Option<RootZone>,
    /// Can units pass over this structure?
    pub passable: bool,
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
    /// The height of the structure, which controls the shadows it casts.
    pub height: u8,
    /// The tiles taken up by this building.
    pub footprint: Option<Footprint>,
    /// The set of tiles that this structure can reach with its roots.
    pub root_zone: Option<RootZone>,
    /// Can units pass over this structure?
    pub passable: bool,
}

impl From<RawStructureData> for StructureData {
    fn from(raw: RawStructureData) -> Self {
        Self {
            organism_variety: raw.organism_variety.map(Into::into),
            kind: raw.kind.into(),
            construction_strategy: raw.construction_strategy.into(),
            max_workers: raw.max_workers,
            height: Height(raw.height as f32),
            footprint: raw.footprint.unwrap_or_default(),
            root_zone: raw.root_zone,
            passable: raw.passable,
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
    /// A structure that can be walked over.
    Path,
    /// A structure that is used to define a special element of the world.
    Landmark,
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
    /// A structure that can be walked over.
    Path,
    /// A structure that is used to define a special element of the world.
    Landmark,
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
            RawStructureKind::Path => Self::Path,
            RawStructureKind::Landmark => Self::Landmark,
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
            .filter(|(id, data)| match &data.organism_variety {
                None => true,
                Some(variety) => variety.prototypical_form == OrganismId::Structure(**id),
            })
            .filter(|(_id, data)| data.kind != StructureKind::Landmark)
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
