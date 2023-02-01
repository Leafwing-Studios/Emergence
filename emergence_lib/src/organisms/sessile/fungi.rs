//! Fungi are structures powered by decomposition.
use crate::{
    self as emergence_lib,
    organisms::{life_cycles::LifeCycle, OrganismType},
};
use bevy::prelude::*;
use emergence_macros::IterableEnum;

use crate::organisms::Species;

use super::SessileBundle;

/// Fungi do not photosynthesize, and instead rely on other sources of energy
#[derive(Component, Default)]
pub struct Fungi;

/// A type of mushroom farmed by leafcutter ants
#[derive(Component, Clone, Default)]
pub struct Leuco;

/// The data needed to spawn a [`Leuco`] [`Fungi`].
#[derive(Bundle)]
pub struct LeucoBundle {
    /// Leuco are fungi
    plant: Fungi,

    /// Fungi are sessile
    sessile_bundle: SessileBundle<Leuco>,
}

impl LeucoBundle {
    /// Creates new [`Leuco`] fungi at specified tile position.
    pub fn new(tile_pos: TilePos) -> Self {
        Self {
            plant: Fungi,
            sessile_bundle: SessileBundle::new(tile_pos),
        }
    }
}

impl Species for Leuco {
    const ORGANISM_TYPE: OrganismType = OrganismType::Fungus;

    type LifeStage = LeucoLifeStage;

    fn life_cycle() -> LifeCycle<Self> {
        // FIXME: add actual life cycles
        LifeCycle {
            life_paths: Default::default(),
        }
    }
}

#[derive(Component, PartialEq, Eq, Default, IterableEnum)]
/// The different life stages of a leuco mushroom
pub enum LeucoLifeStage {
    #[default]
    /// A juvenile leuco mushroom
    Juvenile,
    /// An adult leuco mushroom
    Mature,
    /// A leuco mushroom that ran out of nutrients
    Dead,
}

/// Plugin to handle fungi-specific game logic and simulation.
pub struct FungiPlugin;

impl Plugin for FungiPlugin {
    fn build(&self, _app: &mut App) {
        // TODO; Placeholder for later
    }
}
