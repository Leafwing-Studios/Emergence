//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use crate as emergence_lib;
use crate::simulation::SimulationSet;

use super::clipboard::Tool;
use super::picking::CursorPos;
use super::PlayerAction;
use bevy::prelude::*;
use derive_more::Display;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use emergence_macros::IterableEnum;
use leafwing_abilities::pool::MaxPoolLessThanZero;
use leafwing_abilities::prelude::Pool;
use leafwing_input_manager::prelude::ActionState;
use std::ops::{Div, Mul};

/// Controls, interface and effects of intent-spending abilities.
pub(super) struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (regenerate_intent, use_ability)
                .chain()
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .init_resource::<IntentPool>();
    }
}

/// The different intent-spending "abilities" that the hive mind can use
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, IterableEnum, Display)]
pub(crate) enum IntentAbility {
    /// Gather allied units.
    Lure,
    /// Repel allied units.
    Warning,
    /// Increases the working speed and maintenance costs of structures.
    Flourish,
    /// Decreases the working speed and maintenance costs of structures.
    Fallow,
    /// Increase the signal strength of emitters.
    Amplify,
    /// Decrease the signal strength of emitters.
    Dampen,
}

impl IntentAbility {
    /// The cost of each ability
    fn cost(&self) -> Intent {
        Intent(match self {
            IntentAbility::Lure => 10.,
            IntentAbility::Warning => 20.,
            IntentAbility::Flourish => 30.,
            IntentAbility::Fallow => 30.,
            IntentAbility::Amplify => 10.,
            IntentAbility::Dampen => 10.,
        })
    }
}

/// Uses abilities when pressed at the cursor's location.
fn use_ability(
    cursor_tile_pos: Res<CursorPos>,
    tool: Res<Tool>,
    player_actions: Res<ActionState<PlayerAction>>,
    mut intent_pool: ResMut<IntentPool>,
) {
    let Some(tile_pos) = cursor_tile_pos.maybe_tile_pos() else { return };
    let Tool::Ability(ability) = *tool else { return };

    if player_actions.just_pressed(PlayerAction::UseTool) {
        let cost = ability.cost();
        if intent_pool.current() >= cost {
            intent_pool.expend(cost).unwrap();

            info!("Using {} at {}", ability, tile_pos);
        }
    }
}

/// The amount of Intent available to the player.
/// If they spend it all, they can no longer act.
///
/// This is stored as a single global resource.
#[derive(Debug, Clone, PartialEq, Resource)]
pub(crate) struct IntentPool {
    /// The current amount of available intent.
    current: Intent,
    /// The maximum intent that can be stored.
    max: Intent,
    /// The amount of intent regenerated per second.
    regen_per_second: Intent,
}

/// The maximum amount of intent that can be stored at once
const MAX_INTENT: Intent = Intent(100.);
/// Amount of intent that is regenerated each second
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
pub(crate) struct Intent(pub(crate) f32);

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
fn regenerate_intent(mut intent_pool: ResMut<IntentPool>, time: Res<FixedTime>) {
    if intent_pool.current() != intent_pool.max() {
        intent_pool.regenerate(time.period);
    }
}
