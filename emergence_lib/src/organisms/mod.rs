//! Models organisms, which have two primary types: units (organisms that can move around freely)
//! and structures (organisms that are fixed in place).
use bevy::prelude::*;

use self::{
    sessile::{fungi::FungiPlugin, plants::PlantsPlugin},
    units::UnitsPlugin,
};

pub mod sessile;
pub mod units;

/// All of the compone
#[derive(Bundle, Default)]
pub struct OrganismBundle {
    /// The marker component for orgamisms
    pub organism: Organism,
}

/// A living part of the game ecosystem.
#[derive(Component, Default)]
pub struct Organism;

/// Controls the behavior of living organisms
pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlantsPlugin)
            .add_plugin(FungiPlugin)
            .add_plugin(UnitsPlugin);
    }
}
