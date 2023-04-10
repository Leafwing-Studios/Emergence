//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use crate as emergence_lib;
use crate::signals::{Emitter, SignalModifier};
use crate::simulation::geometry::{MapGeometry, TilePos};
use crate::simulation::SimulationSet;

use super::clipboard::Tool;
use super::picking::CursorPos;
use super::selection::CurrentSelection;
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
        app.add_event::<SignalModifierEvent>()
            .add_systems(
                (regenerate_intent, use_ability)
                    .chain()
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .add_systems(
                (process_signal_modifier_events,)
                    .after(use_ability)
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .init_resource::<IntentPool>();
    }
}

/// The different intent-spending "abilities" that the hive mind can use.
///
/// Note that the order of these variants is important,
/// as it determines the order of the abilities in the UI.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, IterableEnum, Display)]
pub(crate) enum IntentAbility {
    /// Increases the working speed and maintenance costs of structures.
    Flourish,
    /// Repel allied units.
    Warning,
    /// Increase the signal strength of emitters.
    Amplify,
    /// Decreases the working speed and maintenance costs of structures.
    Fallow,
    /// Decrease the signal strength of emitters.
    Dampen,
    /// Gather allied units.
    Lure,
}

impl IntentAbility {
    /// The cost of each ability per second they are used.
    fn cost(&self) -> Intent {
        Intent(match self {
            IntentAbility::Lure => 5.,
            IntentAbility::Warning => 5.,
            IntentAbility::Flourish => 10.,
            IntentAbility::Fallow => 10.,
            IntentAbility::Amplify => 5.,
            IntentAbility::Dampen => 5.,
        })
    }
}

/// Uses abilities when pressed at the cursor's location.
fn use_ability(
    current_selection: Res<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    tool: Res<Tool>,
    player_actions: Res<ActionState<PlayerAction>>,
    mut intent_pool: ResMut<IntentPool>,
    fixed_time: Res<FixedTime>,
    mut signal_modifier_events: EventWriter<SignalModifierEvent>,
) {
    let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
    if relevant_tiles.is_empty() {
        return;
    }
    let Tool::Ability(ability) = *tool else { return };

    if player_actions.pressed(PlayerAction::UseTool) {
        let delta_time = fixed_time.period;
        let n_tiles = relevant_tiles.len();

        let cost = ability.cost() * delta_time.as_secs_f32() * n_tiles as f32;
        if intent_pool.current() >= cost {
            intent_pool.expend(cost).unwrap();

            for &tile_pos in relevant_tiles.selection() {
                match ability {
                    IntentAbility::Lure => todo!(),
                    IntentAbility::Warning => todo!(),
                    IntentAbility::Flourish => todo!(),
                    IntentAbility::Fallow => todo!(),
                    IntentAbility::Amplify => {
                        signal_modifier_events.send(SignalModifierEvent {
                            tile_pos,
                            modifier: SignalModifier::Amplify(delta_time),
                        });
                    }
                    IntentAbility::Dampen => signal_modifier_events.send(SignalModifierEvent {
                        tile_pos,
                        modifier: SignalModifier::Dampen(delta_time),
                    }),
                }
            }
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

/// An event that is sent when a signal modifier is applied to a tile.
#[derive(Clone, Debug)]
struct SignalModifierEvent {
    /// The tile that the modifier is applied to.
    tile_pos: TilePos,
    /// The modifier that is applied.
    modifier: SignalModifier,
}

/// Applies [`SignalModifierEvent`]s, modifying matching [`Emitter`]s.
fn process_signal_modifier_events(
    mut signal_modifier_events: EventReader<SignalModifierEvent>,
    mut emitter_query: Query<&mut Emitter>,
    map_geometry: Res<MapGeometry>,
) {
    for SignalModifierEvent { tile_pos, modifier } in signal_modifier_events.iter() {
        let tile_pos = *tile_pos;
        let modifier = *modifier;

        // FIXME: this won't amplify unit signals, but there's no good way to do that right now without a linear search
        for entity in map_geometry.get_emitters(tile_pos) {
            if let Ok(mut emitter) = emitter_query.get_mut(entity) {
                emitter.modifier += modifier;
            }
        }
    }
}
