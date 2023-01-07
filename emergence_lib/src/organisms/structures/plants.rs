//! Plants are structures powered by photosynthesis.
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    enum_iter::IterableEnum,
    graphics::{organisms::OrganismSprite, sprites::IntoSprite, Tilemap},
    organisms::{Composition, OrganismBundle},
    simulation::pathfinding::Impassable,
};

use super::{Structure, StructureBundle};

/// Plants can photosynthesize
#[derive(Component, Clone)]
pub struct Plant {
    /// Rate at which plants re-generate mass through photosynthesis.
    photosynthesis_rate: f32,
}

impl Plant {
    /// The base rate of photosynthesis
    const PHOTOSYNTHESIS_RATE: f32 = 100.;
}

impl Default for Plant {
    fn default() -> Self {
        Plant {
            photosynthesis_rate: Plant::PHOTOSYNTHESIS_RATE,
        }
    }
}

/// The data needed to make a plant
#[derive(Bundle)]
pub struct PlantBundle {
    /// Data characterizing this plant.
    plant: Plant,
    /// A plant is a structure
    structure_bundle: StructureBundle,
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
    pub fn new(position: TilePos) -> Self {
        Self {
            plant: Plant::default(),
            structure_bundle: StructureBundle {
                structure: Default::default(),
                organism_bundle: OrganismBundle {
                    composition: Composition {
                        mass: Structure::STARTING_MASS,
                    },
                },
            },
            position,
            impassable: Impassable,
        }
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
