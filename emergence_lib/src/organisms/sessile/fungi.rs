//! Fungi are structures powered by decomposition.
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    items::Recipe,
    organisms::Species,
};

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
        let recipe = Recipe::default();

        Self {
            plant: Fungi,
            sessile_bundle: SessileBundle::new(tile_pos, recipe),
        }
    }
}

impl Species for Leuco {
    type LifeStage = LeucoLifeStage;
}

#[derive(Component, PartialEq, Eq, Default)]
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

impl IntoSprite for Leuco {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Organisms
    }

    fn index(&self) -> u32 {
        OrganismSprite::Fungi.index() as u32
    }
}

/// Plugin to handle fungi-specific game logic and simulation.
pub struct FungiPlugin;

impl Plugin for FungiPlugin {
    fn build(&self, _app: &mut App) {
        // TODO; Placeholder for later
    }
}
