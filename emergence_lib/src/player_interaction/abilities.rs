//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use super::cursor::CursorTilePos;
use super::intent::{Intent, IntentPool};
use super::InteractionSystem;
use crate::signals::emitters::Emitter;
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;
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
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

/// Uses abilities when pressed at the cursor's location.
fn use_ability(
    cursor_tile_pos: Res<CursorTilePos>,
    ability_state: Res<ActionState<IntentAbility>>,
    mut intent_pool: ResMut<IntentPool>,
    mut event_writer: EventWriter<SignalModificationEvent>,
) {
    if let Some(pos) = cursor_tile_pos.maybe_tile_pos() {
        for variant in IntentAbility::variants() {
            if ability_state.pressed(variant) {
                if intent_pool.expend(variant.cost()).is_ok() {
                    event_writer.send(SignalModificationEvent::SignalIncrement {
                        emitter: Emitter::Ability(variant),
                        pos,
                        increment: 0.1,
                    })
                };
            }
        }
    }
}
