//! Intent represents the hive mind's ability to act.
//!
//! It slowly recovers over time, with a generous cap,
//! and can be spent on [abilities](super::abilities), [zoning](super::zoning)
//! and in other minor ways to influence the world.

use std::ops::{Div, Mul};

use bevy::{
    prelude::{info, App, IntoSystemDescriptor, Plugin, Res, ResMut, Resource},
    time::Time,
};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use leafwing_abilities::{pool::MaxPoolLessThanZero, prelude::Pool};

use super::InteractionSystem;

pub(super) struct IntentPlugin;

impl Plugin for IntentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntentPool>()
            .add_system(
                regenerate_intent
                    .label(InteractionSystem::ReplenishIntent)
                    .before(InteractionSystem::ApplyZoning)
                    .before(InteractionSystem::UseAbilities),
            )
            .add_system(
                display_intent
                    .after(InteractionSystem::ApplyZoning)
                    .after(InteractionSystem::UseAbilities)
                    .after(InteractionSystem::ReplenishIntent),
            );
    }
}

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

const MAX_INTENT: Intent = Intent(100.);
const INTENT_REGEN: Intent = Intent(10.);

impl Default for IntentPool {
    fn default() -> Self {
        IntentPool {
            current: MAX_INTENT,
            max: MAX_INTENT,
            regen_per_second: INTENT_REGEN,
        }
    }
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

/// Regenerates the [`Intent`] of the hive mind.
///
/// Note that we cannot use the built-in system for this, as our pool is stored somewhat unusually as a resource.
fn regenerate_intent(mut intent_pool: ResMut<IntentPool>, time: Res<Time>) {
    if intent_pool.current() != intent_pool.max() {
        intent_pool.regenerate(time.delta());
    }
}

/// Displays the current quantity of intent stored in the [`IntentPool`].
fn display_intent(intent_pool: Res<IntentPool>) {
    if intent_pool.is_changed() {
        let current = intent_pool.current().0;
        let max = intent_pool.max().0;
        info!("{current} Intent / {max} Intent");
    }
}
