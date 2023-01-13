//! Plants are structures powered by photosynthesis.

use crate::{
    self as emergence_lib,
    items::{ItemCount, ItemId},
};
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use emergence_macros::IterableEnum;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    items::Recipe,
    organisms::Species,
};

use std::{default::Default, time::Duration};

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
    type LifeStage = AcaciaLifeStage;
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

impl IntoSprite for Acacia {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Organisms
    }

    fn index(&self) -> u32 {
        OrganismSprite::Plant.index() as u32
    }
}

impl AcaciaBundle {
    /// Creates new Acacia plant.
    pub fn new(tile_pos: TilePos) -> Self {
        Self {
            plant: Plant,
            sessile_bundle: SessileBundle::new_with_recipe(
                tile_pos,
                Recipe::new(
                    Vec::new(),
                    vec![ItemCount::one(ItemId::acacia_leaf())],
                    Duration::from_secs(10),
                ),
            ),
        }
    }
}

/// Plugin to handle plant-specific game logic and simulation.
pub struct PlantsPlugin;

impl Plugin for PlantsPlugin {
    fn build(&self, _app: &mut App) {}
}
