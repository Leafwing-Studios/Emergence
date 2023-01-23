//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;
use std::fmt::Display;

use self::{
    life_cycles::LifeCycle,
    sessile::{fungi::FungiPlugin, plants::PlantsPlugin},
    units::UnitsPlugin,
};
use crate::enum_iter::IterableEnum;

pub mod life_cycles;
pub mod sessile;
pub mod units;

/// All of the standard components of an [`Organism`]
#[derive(Bundle)]
pub struct OrganismBundle<S: Species> {
    /// The marker component for orgamisms
    pub organism: Organism,
    /// The variety of organism
    pub organism_type: OrganismType,
    /// The marker component for this particular organism
    pub variety: S,
    /// The current life stage for this organism
    pub life_stage: S::LifeStage,
}

impl<S: Species> Default for OrganismBundle<S> {
    fn default() -> Self {
        Self {
            organism: Organism,
            organism_type: S::ORGANISM_TYPE,
            variety: S::default(),
            life_stage: S::LifeStage::default(),
        }
    }
}

/// The type of the organism, e.g. plant or fungus.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum OrganismType {
    /// A plant.
    Plant,

    /// A fungus.
    Fungus,

    /// An ant.
    Ant,
}

impl Display for OrganismType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrganismType::Plant => "Plant",
                OrganismType::Fungus => "Fungus",
                OrganismType::Ant => "Ant",
            }
        )
    }
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// The essential information about a specific variety of organism
///
/// For example, `Acacia` or `Ant` would be a good example of an `Species`,
/// while `Plant` is too general.
pub trait Species: Default + Component {
    /// The [`OrganismType`] associated with this species.
    const ORGANISM_TYPE: OrganismType;

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
