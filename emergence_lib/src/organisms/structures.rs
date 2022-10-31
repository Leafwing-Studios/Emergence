//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, researh, reproduction, storage and more exotic effects.

use crate::organisms::{Composition, OrganismBundle, OrganismType};
use crate::terrain::ImpassableTerrain;
use crate::tiles::IntoTileBundle;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos};

use config::*;
/// Common structure constants
mod config {
    /// The initial mass of spawned structures
    pub const STRUCTURE_STARTING_MASS: f32 = 0.5;
    /// The mass at which sturctures will despawn
    pub const STRUCTURE_DESPAWN_MASS: f32 = 0.01;
    /// The upkeeep cost of each structure
    pub const STRUCTURE_UPKEEP_RATE: f32 = 0.1;
}

/// The data needed to build a structure
#[derive(Bundle, Default)]
pub struct StructureBundle {
    structure: Structure,
    #[bundle]
    organism_bundle: OrganismBundle,
}

/// All structures must pay a cost to keep themselves alive
// TODO: replace with better defaults
#[derive(Component, Clone)]
pub struct Structure {
    upkeep_rate: f32,
    despawn_mass: f32,
}

impl Default for Structure {
    fn default() -> Self {
        Structure {
            upkeep_rate: STRUCTURE_UPKEEP_RATE,
            despawn_mass: STRUCTURE_DESPAWN_MASS,
        }
    }
}

/// Plants can photosynthesize
#[derive(Component, Clone, Default)]
pub struct Plant {
    photosynthesis_rate: f32,
}

/// The data needed to make a plant
#[derive(Bundle, Default)]
pub struct PlantBundle {
    plant: Plant,
    #[bundle]
    structure_bundle: StructureBundle,
    #[bundle]
    tile_bundle: TileBundle,
    impassable: ImpassableTerrain,
}

impl PlantBundle {
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            structure_bundle: StructureBundle {
                structure: Default::default(),
                organism_bundle: OrganismBundle {
                    composition: Composition {
                        mass: STRUCTURE_STARTING_MASS,
                    },
                    ..Default::default()
                },
            },
            tile_bundle: OrganismType::Plant.as_tile_bundle(tilemap_id, position),
            ..Default::default()
        }
    }
}

/// Fungi cannot photosynthesize, and must instead decompose matter
#[derive(Component, Clone, Default)]
pub struct Fungi;

/// The data needed to spawn [`Fungi`].
#[derive(Bundle, Default)]
pub struct FungiBundle {
    fungi: Fungi,
    #[bundle]
    structure_bundle: StructureBundle,
    #[bundle]
    tile_bundle: TileBundle,
}

impl FungiBundle {
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            structure_bundle: StructureBundle {
                organism_bundle: OrganismBundle {
                    ..Default::default()
                },
                ..Default::default()
            },
            tile_bundle: OrganismType::Fungus.as_tile_bundle(tilemap_id, position),
            ..Default::default()
        }
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
fn photosynthesize(time: Res<Time>, mut query: Query<(&Plant, &mut Composition)>) {
    for (plant, mut comp) in query.iter_mut() {
        comp.mass += plant.photosynthesis_rate * time.delta_seconds() * comp.mass.powf(2.0 / 3.0);
    }
}

/// All structures must pay an upkeep cost to sustain the vital functions of life
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
