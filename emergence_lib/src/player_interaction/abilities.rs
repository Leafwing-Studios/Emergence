//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use crate as emergence_lib;

use super::clipboard::Tool;
use super::intent::{Intent, IntentPool};
use super::picking::CursorPos;
use super::{InteractionSystem, PlayerAction};
use bevy::prelude::*;
use derive_more::Display;
use emergence_macros::IterableEnum;
use leafwing_abilities::prelude::Pool;
use leafwing_input_manager::prelude::ActionState;

/// Controls, interface and effects of intent-spending abilities.
pub(super) struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            use_ability
                // If we don't have enough intent, zoning should be applied first to reduce the risk of an error message.
                .after(InteractionSystem::ApplyZoning),
        );
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
