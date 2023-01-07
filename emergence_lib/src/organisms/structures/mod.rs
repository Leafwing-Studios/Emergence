//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.
use crate::graphics::Tilemap;
use crate::organisms::{Composition, OrganismBundle};

use crate::enum_iter::IterableEnum;
use crate::graphics::organisms::OrganismSprite;
use crate::graphics::sprites::IntoSprite;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use self::plants::photosynthesize;

pub mod plants;

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
        OrganismSprite::Fungi.index() as u32
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
