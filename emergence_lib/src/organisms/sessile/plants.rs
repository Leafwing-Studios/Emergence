//! Plants are structures powered by photosynthesis.
use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    items::{ItemCount, ItemId, Recipe},
    organisms::{Composition, OrganismBundle},
    simulation::pathfinding::Impassable,
};

use crate::structures::{crafting::CraftingBundle, Structure, StructureBundle};

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

    /// Rate at which plants re-generate mass through photosynthesis.
    photosynthesis_rate: f32,
}

impl Plant {
    /// The base rate of photosynthesis
    const PHOTOSYNTHESIS_RATE: f32 = 100.;

    /// Create a new plant with the given ID.
    pub fn new(id: PlantId) -> Self {
        Self {
            id,
            photosynthesis_rate: Plant::PHOTOSYNTHESIS_RATE,
        }
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

    /// Plants are impassable
    impassable: Impassable,
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
            organism_bundle: OrganismBundle {
                composition: Composition {
                    mass: Structure::STARTING_MASS,
                },
            },
            crafting_bundle: CraftingBundle::new(crafting_recipe),
            position,
            impassable: Impassable,
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

/// Plants capture energy from the sun
///
/// Photosynthesis scales in proportion to the surface area of plants,
/// and as a result has an allometric scaling ratio of 2.
///
/// A plant's size (in one dimension) is considered to be proportional to the cube root of its mass.
pub fn photosynthesize(time: Res<Time>, mut query: Query<(&Plant, &mut Composition)>) {
    for (plant, mut comp) in query.iter_mut() {
        comp.mass += plant.photosynthesis_rate * time.delta_seconds() * comp.mass.powf(2.0 / 3.0);
    }
}

/// Plugin to handle plant-specific game logic and simulation.
pub struct PlantsPlugin;

impl Plugin for PlantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(photosynthesize);
    }
}
