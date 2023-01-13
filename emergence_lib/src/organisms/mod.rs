//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

use crate::enum_iter::IterableEnum;

use self::{
    life_cycles::LifeCycle,
    sessile::{fungi::FungiPlugin, plants::PlantsPlugin},
    units::UnitsPlugin,
};

pub mod life_cycles;
pub mod organism_details;
pub mod sessile;
pub mod units;

/// All of the standard components of an [`Organism`]
#[derive(Bundle, Default)]
pub struct OrganismBundle<S: Species> {
    /// The marker component for orgamisms
    pub organism: Organism,
    /// The marker component for this particular organism
    pub variety: S,
    /// The current life stage for this organism
    pub life_stage: S::LifeStage,
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// The essential information about a specific variety of organism
///
/// For example, `Acacia` or `Ant` would be a good example of an `Species`,
/// while `Plant` is too general.
pub trait Species: Default + Component {
    /// The enum of possible life stages for this organism
    ///
    /// The [`Default`] implementation should correspond to the life stage of the organism when it is spawned
    type LifeStage: Default + Eq + Component + IterableEnum;

    /// The [`LifeCycle`] and corresponding [`LifePaths`](life_cycles) associated with this species
    fn life_cycle() -> LifeCycle<Self>;
}

/// Controls the behavior of living organisms
pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlantsPlugin)
            .add_plugin(FungiPlugin)
            .add_plugin(UnitsPlugin);
    }
}

/// A trait extension method for [`App`] used to set up generic systems for each species.
pub trait SpeciesExt {
    /// Adds the configuration needed for each species to the [`App`].
    fn add_species<S: Species>(&mut self) -> &mut Self;
}

impl SpeciesExt for App {
    fn add_species<S: Species>(&mut self) -> &mut Self {
        self.init_resource::<LifeCycle<S>>();
        self
    }
}
