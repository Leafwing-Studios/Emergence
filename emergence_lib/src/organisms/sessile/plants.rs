//! Plants are structures powered by photosynthesis.

use crate::{
    self as emergence_lib,
    items::recipe::RecipeId,
    organisms::{life_cycles::LifeCycle, OrganismType},
    simulation::geometry::TilePos,
    structures::StructureId,
};
use bevy::prelude::*;
use emergence_macros::IterableEnum;

use crate::organisms::Species;

use std::default::Default;

use super::SessileBundle;

/// Plants can photosynthesize
#[derive(Component, Default)]
pub struct Plant;

/// Acacia are thorny scrubby plants that rely on ants for protection in exchange for sweet nectar
#[derive(Component, Default, Clone)]
pub struct Acacia;

/// The data needed to make an [`Acacia`] [`Plant`].
#[derive(Bundle)]
pub struct AcaciaBundle {
    /// Acacias are plants
    plant: Plant,
    /// Plants are sessile
    sessile_bundle: SessileBundle<Acacia>,
}

impl Species for Acacia {
    const ORGANISM_TYPE: OrganismType = OrganismType::Plant;

    type LifeStage = AcaciaLifeStage;

    fn life_cycle() -> LifeCycle<Self> {
        // FIXME: add actual life cycles
        LifeCycle {
            life_paths: Default::default(),
        }
    }
}

/// The life stages of an [`Acacia`] plant
#[derive(Component, PartialEq, Eq, Default, IterableEnum)]
pub enum AcaciaLifeStage {
    /// A tiny helpless seedling
    #[default]
    Seedling,
    /// A juvenile plant
    Sprout,
    /// A fully grown plant
    Adult,
    /// A plant that ran out of sun, water or nutrients
    Dead,
}

impl AcaciaBundle {
    /// Creates new Acacia plant.
    pub fn new(tile_pos: TilePos) -> Self {
        Self {
            plant: Plant,
            sessile_bundle: SessileBundle::new_with_recipe(
                tile_pos,
                RecipeId::acacia_leaf_production(),
                StructureId::new("acacia"),
            ),
        }
    }
}

/// Plugin to handle plant-specific game logic and simulation.
pub struct PlantsPlugin;

impl Plugin for PlantsPlugin {
    fn build(&self, _app: &mut App) {}
}
