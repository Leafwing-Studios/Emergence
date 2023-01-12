//! Plants are structures powered by photosynthesis.
use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    items::{ItemCount, ItemId, Recipe},
    organisms::OrganismBundle,
};

use crate::structures::{crafting::CraftingBundle, StructureBundle};

/// The unique identifier of a plant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantId(&'static str);

impl PlantId {
    /// The Acacia plant.
    pub fn acacia() -> Self {
        Self("acacia")
    }
}

/// Plants can photosynthesize
#[derive(Component, Clone)]
pub struct Plant {
    /// The unique identifier of this plant.
    id: PlantId,
}

impl Plant {
    /// Create a new plant with the given ID.
    pub fn new(id: PlantId) -> Self {
        Self { id }
    }

    /// The unique identifier of this plant.
    pub fn id(&self) -> &PlantId {
        &self.id
    }
}

/// The data needed to make a plant
#[derive(Bundle)]
pub struct PlantBundle {
    /// Data characterizing this plant.
    plant: Plant,

    /// A plant is an organism
    organism_bundle: OrganismBundle,

    /// A plant is a structure
    structure_bundle: StructureBundle,

    /// A plant can craft things
    crafting_bundle: CraftingBundle,

    /// Position in the world
    position: TilePos,
}

impl IntoSprite for Plant {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Organisms
    }

    fn index(&self) -> u32 {
        OrganismSprite::Plant.index() as u32
    }
}

impl PlantBundle {
    /// Creates new plant at specified tile position, in the specified tilemap.
    pub fn new(id: PlantId, crafting_recipe: Recipe, position: TilePos) -> Self {
        Self {
            plant: Plant::new(id),
            structure_bundle: StructureBundle::default(),
            organism_bundle: OrganismBundle::default(),
            crafting_bundle: CraftingBundle::new(crafting_recipe),
            position,
        }
    }

    /// Create a new Acacia plant.
    pub fn acacia(position: TilePos) -> Self {
        Self::new(
            PlantId::acacia(),
            Recipe::new(
                Vec::new(),
                vec![ItemCount::one(ItemId::acacia_leaf())],
                Duration::from_secs(10),
            ),
            position,
        )
    }
}

/// Plugin to handle plant-specific game logic and simulation.
pub struct PlantsPlugin;

impl Plugin for PlantsPlugin {
    fn build(&self, _app: &mut App) {}
}
