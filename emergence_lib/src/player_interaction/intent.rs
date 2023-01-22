//! Intent represents the hive mind's ability to act.
//!
//! It slowly recovers over time, with a generous cap,
//! and can be spent on [abilities](super::abilities), [zoning](super::zoning)
//! and in other minor ways to influence the world.

use std::ops::{Div, Mul};

use bevy::prelude::Resource;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use leafwing_abilities::{pool::MaxPoolLessThanZero, prelude::Pool};

/// The amount of Intent available to the player.
/// If they spend it all, they can no longer act.
///
/// This is stored as a single global resource.
#[derive(Debug, Clone, PartialEq, Resource)]
pub struct IntentPool {
    /// The current amount of available intent.
    current: Intent,
    /// The maximum intent that can be stored.
    max: Intent,
    /// The amount of intent regenerated per second.
    pub regen_per_second: Intent,
}

/// A quantity of Intent, used to modify an [`IntentPool`].
///
/// This is used to measure the amount of Intent that must be spent to perform various actions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Add, Sub, AddAssign, SubAssign)]
pub struct Intent(pub f32);

impl Mul<f32> for Intent {
    type Output = Intent;

    fn mul(self, rhs: f32) -> Intent {
        Intent(self.0 * rhs)
    }
}

impl Div<f32> for Intent {
    type Output = Intent;

    fn div(self, rhs: f32) -> Intent {
        Intent(self.0 / rhs)
    }
}

impl Pool for IntentPool {
    type Quantity = Intent;
    const ZERO: Intent = Intent(0.);

    fn new(current: Self::Quantity, max: Self::Quantity, regen_per_second: Self::Quantity) -> Self {
        IntentPool {
            current,
            max,
            regen_per_second,
        }
    }

    fn current(&self) -> Self::Quantity {
        self.current
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
        let actual_value = Intent(new_quantity.0.clamp(0., self.max.0));
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
