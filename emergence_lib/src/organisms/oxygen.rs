//! Organisms that are underwater should eventually drown and die.
use bevy::prelude::*;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use leafwing_abilities::{pool::MaxPoolLessThanMin, prelude::Pool};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Div, Mul},
};

use crate::{
    asset_management::manifest::Id,
    geometry::{Height, MapGeometry, VoxelPos},
    structures::{commands::StructureCommandsExt, Footprint},
    units::unit_manifest::Unit,
    water::WaterDepth,
};

use super::Organism;

/// The amount of oxygen available to an organism.
/// If they run out, they die.
#[derive(Debug, Clone, PartialEq, Component, Resource, Serialize, Deserialize)]
pub struct OxygenPool {
    /// The current amount of stored oxygen.
    current: Oxygen,
    /// The point at which the organism begins to panic and seek oxygen.
    panic_threshold: Oxygen,
    /// The maximum oxygen that can be stored.
    max: Oxygen,
}

impl OxygenPool {
    /// Construct a new full oxygen pool with a max oxygen of `max`.
    pub fn new(max: Oxygen, panic_threshold: f32) -> Self {
        OxygenPool {
            current: max,
            panic_threshold: panic_threshold * max,
            max,
        }
    }

    /// Is this organism out of oxygen?
    pub(crate) fn is_empty(&self) -> bool {
        self.current <= Oxygen(0.)
    }

    /// Is this organism full on oxygen?
    pub(crate) fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Should this organism be panicking?
    ///
    /// An organism panics when it is below the panic threshold.
    pub(crate) fn should_panic(&self) -> bool {
        self.current <= self.panic_threshold
    }
}

impl Display for OxygenPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current, self.max)
    }
}

/// A quantity of oxygen, used to modify a [`OxygenPool`].
///
/// Organisms produce oxygen while above the water, and lose it while underwater.
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
    const MIN: Oxygen = Oxygen(0.);

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

    fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
        if new_max < Self::MIN {
            Err(MaxPoolLessThanMin)
        } else {
            self.max = new_max;
            self.set_current(self.current);
            Ok(())
        }
    }
}

/// Increases and decreases oxygen levels over time, and kills all organisms that run out of oxygen.
pub(super) fn manage_oxygen(
    mut unit_query: Query<(Entity, &VoxelPos, &mut OxygenPool), With<Id<Unit>>>,
    mut structure_query: Query<
        (&VoxelPos, &Footprint, &mut OxygenPool),
        (Without<Id<Unit>>, With<Organism>),
    >,
    water_depth_query: Query<&WaterDepth>,
    time: Res<Time>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    let delta_time = time.delta().as_secs_f32();

    for (entity, &voxel_pos, mut oxygen_pool) in unit_query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
        let surface_water_depth = water_depth_query
            .get(terrain_entity)
            .unwrap()
            .surface_water_depth();

        if surface_water_depth > Height::WADING_DEPTH {
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

    for (&voxel_pos, footprint, mut oxygen_pool) in structure_query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
        let surface_water_depth = water_depth_query
            .get(terrain_entity)
            .unwrap()
            .surface_water_depth();

        if surface_water_depth > footprint.max_height().into() {
            let proposed = oxygen_pool.current - Oxygen::CONSUMPTION_RATE * delta_time;
            oxygen_pool.set_current(proposed);

            if oxygen_pool.is_empty() {
                commands.despawn_structure(voxel_pos);
            }
        } else {
            let proposed = oxygen_pool.current + Oxygen::REGEN_RATE * delta_time;
            oxygen_pool.set_current(proposed);
        }
    }
}
