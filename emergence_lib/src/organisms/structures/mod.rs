//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.
use crate::organisms::{Composition, OrganismBundle};

use bevy::prelude::*;

use self::{crafting::CraftingPlugin, fungi::FungiPlugin, plants::PlantsPlugin};

mod crafting;
pub mod fungi;
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

/// The systems that make structures tick.
pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlantsPlugin)
            .add_plugin(FungiPlugin)
            .add_plugin(CraftingPlugin)
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
