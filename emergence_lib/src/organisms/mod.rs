//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

use self::{
    life_cycles::LifeCycle,
    sessile::{fungi::FungiPlugin, plants::PlantsPlugin},
    units::UnitsPlugin,
};

pub mod life_cycles;
pub mod sessile;
pub mod units;

/// All of the compone
#[derive(Bundle, Default)]
pub struct OrganismBundle<O: OrganismKind> {
    /// The marker component for orgamisms
    pub organism: Organism,
    /// The marker component for this particular organism
    pub variety: O,
    /// The current life stage and life paths for this organism
    pub life_cycle: LifeCycle<O>,
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// The essential information about a specific variety of organism
///
/// For example, `Acacia` or `Ant` would be a good example of an `OrganismKind`,
/// while `Plant` is too general.
pub trait OrganismKind: Default + Component {
    /// The enum of possible life stages for this organism
    ///
    /// The [`Default`] implementation should correspond to the life stage of the organism when it is spawned
    type LifeStage: Default + Eq + Send + Sync + 'static;
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
