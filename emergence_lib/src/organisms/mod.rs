//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

use crate::enum_iter::IterableEnum;

use self::{
    organism_details::DetailsPlugin,
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
}

/// Controls the behavior of living organisms
pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlantsPlugin)
            .add_plugin(FungiPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(DetailsPlugin);
    }
}
