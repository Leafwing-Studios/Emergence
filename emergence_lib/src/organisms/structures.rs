//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.
use crate::graphics::Tilemap;
use crate::organisms::{Composition, OrganismBundle};

use crate::enum_iter::IterableEnum;
use crate::graphics::organisms::OrganismSpriteIndex;
use crate::graphics::sprites::IntoSprite;
use crate::simulation::pathfinding::PathfindingImpassable;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

/// The data needed to build a structure
#[derive(Bundle, Default)]
pub struct StructureBundle {
    /// Data characterizing structures
    structure: Structure,
    /// Structures are organisms (for now)
    organism_bundle: OrganismBundle,
}

/// All structures must pay a cost to keep themselves alive
#[derive(Component, Clone)]
pub struct Structure {
    /// Mass cost per tick to stay alive
    upkeep_rate: f32,
    /// Mass at which the structure will be despawned
    despawn_mass: f32,
}

impl Structure {
    /// The initial mass of spawned structures
    pub const STARTING_MASS: f32 = 0.5;
    /// The mass at which structures will despawn
    pub const DESPAWN_MASS: f32 = 0.01;
    /// The upkeep cost of each structure, relative to its total mass
    pub const UPKEEP_RATE: f32 = 0.1;
}

impl Default for Structure {
    fn default() -> Self {
        Structure {
            upkeep_rate: Structure::UPKEEP_RATE,
            despawn_mass: Structure::DESPAWN_MASS,
        }
    }
}

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
    impassable: PathfindingImpassable,
}

impl IntoSprite for Plant {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Organisms
    }

    fn index(&self) -> u32 {
        OrganismSpriteIndex::Plant.index() as u32
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
                    ..Default::default()
                },
            },
            position,
            impassable: PathfindingImpassable,
        }
    }
}

/// Fungi cannot photosynthesize, and must instead decompose matter
#[derive(Component, Clone, Default)]
pub struct Fungi;

/// The data needed to spawn [`Fungi`].
#[derive(Bundle)]
pub struct FungiBundle {
    /// Data characterizing fungi
    fungi: Fungi,
    /// Fungi are structures.
    structure_bundle: StructureBundle,
    /// Data needed to visually represent this fungus.
    /// Position in the world
    position: TilePos,
}

impl FungiBundle {
    /// Creates new fungi at specified tile position, in the specified tilemap.
    pub fn new(position: TilePos) -> Self {
        Self {
            fungi: Fungi,
            structure_bundle: StructureBundle {
                organism_bundle: OrganismBundle {
                    ..Default::default()
                },
                ..Default::default()
            },
            position,
        }
    }
}

impl IntoSprite for Fungi {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Organisms
    }

    fn index(&self) -> u32 {
        OrganismSpriteIndex::Fungi.index() as u32
    }
}

/// The systems that make structures tick.
pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(photosynthesize)
            .add_system(upkeep)
            .add_system(cleanup);
    }
}

/// Plants capture energy from the sun
///
/// Photosynthesis scales in proportion to the surface area of plants,
/// and as a result has an allometric scaling ratio of 2.
///
/// A plant's size (in one dimension) is considered to be proportional to the cube root of its mass.
fn photosynthesize(time: Res<Time>, mut query: Query<(&Plant, &mut Composition)>) {
    for (plant, mut comp) in query.iter_mut() {
        comp.mass += plant.photosynthesis_rate * time.delta_seconds() * comp.mass.powf(2.0 / 3.0);
    }
}

/// All structures must pay an upkeep cost to sustain the vital functions of life.
///
/// Maintenance of biological functions is proportional to the mass that must be maintained.
fn upkeep(time: Res<Time>, mut query: Query<(&Structure, &mut Composition)>) {
    for (structure, mut comp) in query.iter_mut() {
        comp.mass -= structure.upkeep_rate * time.delta_seconds() * comp.mass;
    }
}

/// If structures grow too weak, they die and are despawned.
fn cleanup(mut commands: Commands, query: Query<(&Structure, Entity, &Composition)>) {
    for (structure, ent, comp) in query.iter() {
        if comp.mass <= structure.despawn_mass {
            commands.entity(ent).despawn();
        }
    }
}
