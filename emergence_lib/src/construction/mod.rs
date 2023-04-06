//! Tools and systems for constructing structures and terraforming the world.

use bevy::utils::{Duration, HashMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::crafting::components::InputInventory;
use crate::items::slot::ItemSlot;
use crate::terrain::terrain_manifest::Terrain;
use crate::{asset_management::manifest::Id, structures::structure_manifest::Structure};

pub(crate) mod demolition;
pub(crate) mod ghosts;
pub(crate) mod terraform;
pub(crate) mod zoning;

/// Systems and resources for constructing structures and terraforming the world.
pub(crate) struct ConstructionPlugin;

impl Plugin for ConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ghosts::GhostPlugin)
            .add_plugin(terraform::TerraformingPlugin)
            .add_plugin(zoning::ZoningPlugin);
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
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionData {
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`None`], no work will be needed at all.
    pub work: Option<Duration>,
    /// The set of items needed to create a new copy of this structure
    pub materials: InputInventory,
    /// The set of terrain types that this structure can be built on
    pub allowed_terrain_types: AllowedTerrainTypes,
}

/// The set of terrain types that this structure can be built on.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum AllowedTerrainTypes {
    /// Any terrain type is allowed.
    #[default]
    Any,
    /// Only the provided terrain types are allowed.
    Only(HashSet<Id<Terrain>>),
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
        materials: HashMap<String, u32>,
        /// The set of terrain types that this structure can be built on
        ///
        /// If this is empty, any terrain type is allowed.
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

                let materials = InputInventory::Exact { inventory };
                ConstructionStrategy::Direct(ConstructionData {
                    work: work.map(Duration::from_secs_f32),
                    materials,
                    allowed_terrain_types: if allowed_terrain_types.is_empty() {
                        AllowedTerrainTypes::Any
                    } else {
                        AllowedTerrainTypes::Only(
                            allowed_terrain_types
                                .into_iter()
                                .map(Id::from_name)
                                .collect(),
                        )
                    },
                })
            }
        }
    }
}
