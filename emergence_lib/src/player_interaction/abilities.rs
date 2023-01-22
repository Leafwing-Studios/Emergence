//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use super::cursor::CursorTilePos;
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::{Lure, Warning};
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Provides the interface between the player and the hive.
pub struct HiveMindPlugin;

impl Plugin for HiveMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<IntentAbility>::default())
            .add_startup_system(initialize_hive_mind)
            .add_system(use_ability);
    }
}

/// Represents the interface between the player and the hive.
#[derive(Component, Clone, Copy)]
pub struct HiveMind;

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
}

/// Startup system initializing the [`HiveMind`].
fn initialize_hive_mind(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(HiveMind)
        .insert(InputManagerBundle::<IntentAbility> {
            input_map: IntentAbility::default_input_map(),
            ..default()
        });
}

/// Marks the tile the mouse is over top of with  if the mouse is hovered over a hex tile.
fn use_ability(
    mut signal_create_evw: EventWriter<SignalModificationEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
    hive_mind_query: Query<&ActionState<IntentAbility>, With<HiveMind>>,
) {
    let hive_mind_state = hive_mind_query.single();

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
