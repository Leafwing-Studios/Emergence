//! Organisms that are underwater should eventually drown and die.
use bevy::prelude::*;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use leafwing_abilities::{pool::MaxPoolLessThanZero, prelude::Pool};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Div, Mul},
};

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, TilePos},
    structures::{commands::StructureCommandsExt, structure_manifest::Structure},
    units::unit_manifest::Unit,
    water::WaterTable,
};

use super::Organism;

/// The amount of energy available to an organism.
/// If they run out, they die.
#[derive(Debug, Clone, PartialEq, Component, Resource, Serialize, Deserialize)]
pub struct OxygenPool {
    /// The current amount of stored oxygen.
    current: Oxygen,
    /// The maximum oxygen that can be stored.
    max: Oxygen,
}

impl OxygenPool {
    /// Construct a new oxygen pool with a max energy of `max` and a rate of `regen_per_second`.
    pub fn new(max: Oxygen) -> Self {
        OxygenPool { current: max, max }
    }

    /// Is this organism out of oxygen?
    pub(crate) fn is_empty(&self) -> bool {
        self.current <= Oxygen(0.)
    }
}

impl Display for OxygenPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current, self.max)
    }
}

/// A quantity of energy, used to modify a [`EnergyPool`].
///
/// Organisms produce energy by crafting recipes.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Default,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Serialize,
    Deserialize,
)]
pub struct Oxygen(pub f32);

impl Oxygen {
    /// The rate at which oxygen is consumed by an organism.
    pub const CONSUMPTION_RATE: Oxygen = Oxygen(20.);

    /// The rate at which oxygen is regenerated by an organism.
    pub const REGEN_RATE: Oxygen = Oxygen(50.);

    /// The standard amount of oxygen an organism can store.
    pub const STANDARD_MAX: Oxygen = Oxygen(100.);
}

impl Display for Oxygen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

impl Mul<f32> for Oxygen {
    type Output = Oxygen;

    fn mul(self, rhs: f32) -> Oxygen {
        Oxygen(self.0 * rhs)
    }
}

impl Mul<Oxygen> for f32 {
    type Output = Oxygen;

    fn mul(self, rhs: Oxygen) -> Oxygen {
        Oxygen(self * rhs.0)
    }
}

impl Div<f32> for Oxygen {
    type Output = Oxygen;

    fn div(self, rhs: f32) -> Oxygen {
        Oxygen(self.0 / rhs)
    }
}

impl Pool for OxygenPool {
    type Quantity = Oxygen;
    const ZERO: Oxygen = Oxygen(0.);

    fn new(
        current: Self::Quantity,
        max: Self::Quantity,
        _regen_per_second: Self::Quantity,
    ) -> Self {
        OxygenPool { current, max }
    }

    fn current(&self) -> Self::Quantity {
        self.current
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
        let actual_value = Oxygen(new_quantity.0.clamp(0., self.max.0));
        self.current = actual_value;
        self.current
    }

    fn max(&self) -> Self::Quantity {
        self.max
    }

    fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanZero> {
        if new_max < Self::ZERO {
            Err(MaxPoolLessThanZero)
        } else {
            self.max = new_max;
            self.set_current(self.current);
            Ok(())
        }
    }

    fn regen_per_second(&self) -> Self::Quantity {
        Oxygen::REGEN_RATE
    }

    fn set_regen_per_second(&mut self, _new_regen_per_second: Self::Quantity) {
        panic!("Cannot set regen per second for oxygen pool.")
    }
}

/// Increases and decreases oxygen levels over time, and kills all organisms that run out of oxygen.
pub(super) fn manage_oxygen(
    mut unit_query: Query<(Entity, &TilePos, &mut OxygenPool), With<Id<Unit>>>,
    mut structure_query: Query<
        (&TilePos, &mut OxygenPool),
        (With<Id<Structure>>, Without<Id<Unit>>, With<Organism>),
    >,
    water_table: Res<WaterTable>,
    fixed_time: Res<FixedTime>,
    mut commands: Commands,
) {
    let delta_time = fixed_time.period.as_secs_f32();

    for (entity, &tile_pos, mut oxygen_pool) in unit_query.iter_mut() {
        let water_depth = water_table.surface_water_depth(tile_pos);
        if water_depth > Height::WADING_DEPTH {
            let proposed = oxygen_pool.current - Oxygen::CONSUMPTION_RATE * delta_time;
            oxygen_pool.set_current(proposed);

            if oxygen_pool.is_empty() {
                commands.entity(entity).despawn_recursive();
            }
        } else {
            let proposed = oxygen_pool.current + Oxygen::REGEN_RATE * delta_time;
            oxygen_pool.set_current(proposed);
        }
    }

    for (&tile_pos, mut oxygen_pool) in structure_query.iter_mut() {
        let water_depth = water_table.surface_water_depth(tile_pos);
        if water_depth > Height::WADING_DEPTH {
            let proposed = oxygen_pool.current - Oxygen::CONSUMPTION_RATE * delta_time;
            oxygen_pool.set_current(proposed);

            if oxygen_pool.is_empty() {
                commands.despawn_structure(tile_pos);
            }
        } else {
            let proposed = oxygen_pool.current + Oxygen::REGEN_RATE * delta_time;
            oxygen_pool.set_current(proposed);
        }
    }
}
