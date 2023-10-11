//! Tools and systems for constructing structures and terraforming the world.

use bevy::utils::{Duration, HashMap};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::crafting::inventories::InputInventory;
use crate::items::slot::ItemSlot;
use crate::simulation::SimulationSet;
use crate::{asset_management::manifest::Id, structures::structure_manifest::Structure};

use self::demolition::set_emitter_for_structures_to_be_demolished;
use self::terraform::{terraforming_lifecycle, terraforming_signals};

pub(crate) mod demolition;
pub(crate) mod ghosts;
pub(crate) mod terraform;
pub(crate) mod zoning;

/// Systems and resources for constructing structures and terraforming the world.
pub(crate) struct ConstructionPlugin;

impl Plugin for ConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ghosts::GhostPlugin)
            .add_plugin(zoning::ZoningPlugin)
            // Must run after crafting emitters in order to wipe out their signals
            .add_systems(
                FixedUpdate,
                set_emitter_for_structures_to_be_demolished
                    .after(crate::crafting::set_crafting_emitter)
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .add_systems(
                FixedUpdate,
                (terraforming_lifecycle, terraforming_signals)
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
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
    /// A landmark, which cannot be built.
    Landmark,
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
    },
    /// A landmark, which cannot be built.
    Landmark,
}

impl From<RawConstructionStrategy> for ConstructionStrategy {
    fn from(raw: RawConstructionStrategy) -> Self {
        match raw {
            RawConstructionStrategy::Seedling(seedling_name) => {
                ConstructionStrategy::Seedling(Id::from_name(seedling_name))
            }
            RawConstructionStrategy::Direct { work, materials } => {
                let inventory = materials
                    .into_iter()
                    .map(|(item_name, count)| ItemSlot::empty(Id::from_name(item_name), count))
                    .collect();

                let materials = InputInventory::Exact { inventory };
                ConstructionStrategy::Direct(ConstructionData {
                    work: work.map(Duration::from_secs_f32),
                    materials,
                })
            }
            RawConstructionStrategy::Landmark => ConstructionStrategy::Landmark,
        }
    }
}
