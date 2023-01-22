//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use super::cursor::CursorTilePos;
use super::intent::Intent;
use super::InteractionSystem;
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::{Lure, Warning};
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Controls, interface and effects of intent-spending abilities.
pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<IntentAbility>::default())
            .init_resource::<ActionState<IntentAbility>>()
            .insert_resource(IntentAbility::default_input_map())
            .add_system(
                use_ability
                    .label(InteractionSystem::UseAbilities)
                    // If we don't have enough intent, zoning should be applied first to reduce the risk of an error message.
                    .after(InteractionSystem::ApplyZoning),
            );
    }
}

/// The different intent-spending "abilities" that the hive mind can use
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
pub enum IntentAbility {
    /// Gather allied units.
    Lure,
    /// Repel allied units.
    Warning,
}

impl IntentAbility {
    /// The starting keybinds
    fn default_input_map() -> InputMap<IntentAbility> {
        InputMap::new([
            (KeyCode::F, IntentAbility::Lure),
            (KeyCode::G, IntentAbility::Warning),
        ])
    }

    /// The cost of each ability
    fn cost(&self) -> Intent {
        match self {
            IntentAbility::Lure => Intent(10.),
            IntentAbility::Warning => Intent(20.),
        }
    }
}

/// Marks the tile the mouse is over top of with  if the mouse is hovered over a hex tile.
fn use_ability(
    mut signal_create_evw: EventWriter<SignalModificationEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
    hive_mind_state: Res<ActionState<IntentAbility>>,
) {
    if hive_mind_state.pressed(IntentAbility::Lure) {
        if let Some(pos) = cursor_tile_pos.maybe_tile_pos() {
            signal_create_evw.send(SignalModificationEvent::SignalIncrement {
                emitter: Emitter::Stock(Lure),
                pos,
                increment: 0.1,
            })
        }
    }
    // TODO: Fix the failing clamp in curves
    if hive_mind_state.pressed(IntentAbility::Warning) {
        if let Some(pos) = cursor_tile_pos.maybe_tile_pos() {
            signal_create_evw.send(SignalModificationEvent::SignalIncrement {
                emitter: Emitter::Stock(Warning),
                pos,
                increment: 0.01,
            });
        }
    }
}
