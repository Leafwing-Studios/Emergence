//! Logic and data types for energy.

use bevy::prelude::*;
use bevy::utils::Duration;
use core::fmt::Display;
use core::ops::{Add, Div, Mul};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use leafwing_abilities::{pool::MaxPoolLessThanZero, prelude::Pool};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

use crate::asset_management::manifest::Id;
use crate::simulation::geometry::MapGeometry;
use crate::structures::structure_manifest::Structure;
use crate::{simulation::geometry::TilePos, structures::commands::StructureCommandsExt};

/// The amount of energy available to an organism.
/// If they run out, they die.
#[derive(Debug, Clone, PartialEq, Component, Resource, Serialize, Deserialize)]
pub struct EnergyPool {
    /// The current amount of stored energy.
    current: Energy,
    /// The maximum energy that can be stored.
    max: Energy,
    /// The threshold at which desperate action is taken to gain more energy.
    warning_threshold: Energy,
    /// The threshold at which no more action is taken to gain energy.
    satiation_threshold: Energy,
    /// The amount of life regenerated per second.
    pub(crate) regen_per_second: Energy,
}

impl EnergyPool {
    /// Quickly construct a new empty energy pool with a max energy of `max` and no regeneration.
    pub fn simple(max: f32) -> Self {
        EnergyPool::new_empty(Energy(max), Energy(0.))
    }

    /// Is this organism out of energy?
    pub(crate) fn is_empty(&self) -> bool {
        self.current <= Energy(0.)
    }

    /// Is this organism close to running out of energy?
    pub(crate) fn is_hungry(&self) -> bool {
        self.current <= self.warning_threshold
    }

    /// Is this organism's appetite satisfied?
    pub(crate) fn is_satiated(&self) -> bool {
        self.current >= self.satiation_threshold
    }

    /// Is this pool full?
    pub(crate) fn is_full(&self) -> bool {
        self.current >= self.max
    }
}

impl Display for EnergyPool {
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
pub struct Energy(pub f32);

impl Display for Energy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

impl Mul<f32> for Energy {
    type Output = Energy;

    fn mul(self, rhs: f32) -> Energy {
        Energy(self.0 * rhs)
    }
}

impl Mul<Energy> for f32 {
    type Output = Energy;

    fn mul(self, rhs: Energy) -> Energy {
        Energy(self * rhs.0)
    }
}

impl Div<f32> for Energy {
    type Output = Energy;

    fn div(self, rhs: f32) -> Energy {
        Energy(self.0 / rhs)
    }
}

impl Pool for EnergyPool {
    type Quantity = Energy;
    const ZERO: Energy = Energy(0.);

    fn new(current: Self::Quantity, max: Self::Quantity, regen_per_second: Self::Quantity) -> Self {
        // TODO: don't hard code this.
        // Blocked on: https://github.com/Leafwing-Studios/leafwing_abilities/issues/18
        let warning_threshold = 0.25 * max;
        let satiation_threshold = 0.75 * max;

        EnergyPool {
            current,
            warning_threshold,
            satiation_threshold,
            max,
            regen_per_second,
        }
    }

    fn current(&self) -> Self::Quantity {
        self.current
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
        let actual_value = Energy(new_quantity.0.clamp(0., self.max.0));
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
        self.regen_per_second
    }

    fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity) {
        self.regen_per_second = new_regen_per_second;
    }
}

/// Consumes [`Energy`] over time, taking into account the tile's [`VigorModifier`].
pub(super) fn consume_energy(
    fixed_time: Res<FixedTime>,
    mut energy_query: Query<(&mut EnergyPool, &TilePos)>,
    vigor_modifier_query: Query<&VigorModifier>,
    map_geometry: Res<MapGeometry>,
) {
    let delta_time = fixed_time.period.as_secs_f32();

    for (mut energy_pool, &tile_pos) in energy_query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let vigor_modifier = vigor_modifier_query.get(terrain_entity).unwrap();

        let vigor_multiplier = match vigor_modifier {
            VigorModifier::None => 1.,
            VigorModifier::Flourish(..) => VigorModifier::RATIO,
            VigorModifier::Fallow(..) => 1. / VigorModifier::RATIO,
        };
        // Note that regen rates are almost always negative.
        let regen_rate = energy_pool.regen_per_second * vigor_multiplier;
        let current = energy_pool.current();

        energy_pool.set_current(current + regen_rate * delta_time);
    }
}

/// Despawns organisms when they run out of energy
pub(super) fn kill_organisms_when_out_of_energy(
    organism_query: Query<(Entity, &EnergyPool, &TilePos, Option<&Id<Structure>>)>,
    mut commands: Commands,
) {
    for (entity, energy_pool, tile_pos, maybe_structure) in organism_query.iter() {
        if energy_pool.is_empty() {
            match maybe_structure {
                Some(_) => commands.despawn_structure(*tile_pos),
                None => commands.entity(entity).despawn_recursive(),
            }
        }
    }
}

/// Modifies the working speed and energy consumption rate of an organism.
///
/// This is stored as a component on each tile, and is applied to all entities at that tile position.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VigorModifier {
    /// No modifier is applied.
    #[default]
    None,
    /// Working speed and energy consumption is multiplied for the duration of this effect.
    Flourish(Duration),
    /// Working speed and energy consumption is divided for the duration of this effect.
    Fallow(Duration),
}

impl VigorModifier {
    /// Controls the ratio of the signal strength when modified.
    ///
    /// This should be greater than 1.
    const RATIO: f32 = 10.;
}

impl Display for VigorModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VigorModifier::None => write!(f, "None"),
            VigorModifier::Flourish(duration) => write!(f, "Flourish({})", duration.as_secs_f32()),
            VigorModifier::Fallow(duration) => write!(f, "Fallow({})", duration.as_secs_f32()),
        }
    }
}

impl Add for VigorModifier {
    type Output = Self;

    /// Combines two modifiers, returning the resulting modifier.
    ///
    /// If the two modifiers are the same, their durations are added together.
    /// If the two modifiers are different, they cancel each other out.
    fn add(self, other: VigorModifier) -> Self {
        match (self, other) {
            (VigorModifier::None, other) | (other, VigorModifier::None) => other,
            (
                VigorModifier::Flourish(amplify_duration),
                VigorModifier::Fallow(diminish_duration),
            )
            | (
                VigorModifier::Fallow(diminish_duration),
                VigorModifier::Flourish(amplify_duration),
            ) => {
                if amplify_duration == diminish_duration {
                    VigorModifier::None
                } else if amplify_duration > diminish_duration {
                    VigorModifier::Flourish(amplify_duration - diminish_duration)
                } else {
                    VigorModifier::Fallow(diminish_duration - amplify_duration)
                }
            }
            (VigorModifier::Flourish(a), VigorModifier::Flourish(b)) => {
                VigorModifier::Flourish(a + b)
            }
            (VigorModifier::Fallow(a), VigorModifier::Fallow(b)) => VigorModifier::Fallow(a + b),
        }
    }
}

impl AddAssign for VigorModifier {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

/// Ticks down the duration of all [`VigorModifier`] components.
pub(super) fn tick_vigor_modifiers(
    mut query: Query<&mut VigorModifier>,
    fixed_time: Res<FixedTime>,
) {
    let delta_time = fixed_time.period;

    for mut vigor_modifier in query.iter_mut() {
        match *vigor_modifier {
            VigorModifier::None => {}
            VigorModifier::Flourish(duration) => {
                if duration > delta_time {
                    *vigor_modifier = VigorModifier::Flourish(duration - delta_time);
                } else {
                    *vigor_modifier = VigorModifier::None;
                }
            }
            VigorModifier::Fallow(duration) => {
                if duration > delta_time {
                    *vigor_modifier = VigorModifier::Fallow(duration - delta_time);
                } else {
                    *vigor_modifier = VigorModifier::None;
                }
            }
        }
    }
}
