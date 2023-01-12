//! Fungi are structures powered by decomposition.
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    organisms::OrganismBundle,
    structures::StructureBundle,
};

/// Fungi cannot photosynthesize, and must instead decompose matter
#[derive(Component, Clone, Default)]
pub struct Fungi;

/// The data needed to spawn [`Fungi`].
#[derive(Bundle)]
pub struct FungiBundle {
    /// Data characterizing fungi
    fungi: Fungi,
    /// Fungi are organisms
    organism_bundle: OrganismBundle,
    /// Fungi are structures.
    structure_bundle: StructureBundle,
    /// Position in the world
    position: TilePos,
}

impl FungiBundle {
    /// Creates new fungi at specified tile position, in the specified tilemap.
    pub fn new(position: TilePos) -> Self {
        Self {
            fungi: Fungi,
            structure_bundle: StructureBundle::default(),
            organism_bundle: OrganismBundle::default(),
            position,
        }
    }
}

impl IntoSprite for Fungi {
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
