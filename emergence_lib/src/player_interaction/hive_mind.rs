//! Represents the player.

use super::cursor::CursorTilePos;
use super::tile_selection::{SelectedTiles, TileSelectionAction};
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::{PheromoneAttract, PheromoneRepulse};
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Provides the interface between the player and the hive.
pub struct HiveMindPlugin;

impl Plugin for HiveMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<HiveMindAction>::default())
            .add_startup_system(initialize_hive_mind)
            .add_system(place_pheromone);
    }
}

/// Represents the interface between the player and the hive.
#[derive(Component, Clone, Copy)]
pub struct HiveMind;

/// Enumerates the actions a hive mind (the player) can take.
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HiveMindAction {
    /// Place an attractive pheromone.
    PlaceAttractivePheromone,
    /// Place a repulsive pheromone.
    PlaceRepulsivePheromone,
}

// TODO: rework this to use LWIM conventions for mapping controls
/// Interface for player controls
pub struct HiveMindControls {
    /// Place an attractive pheromone
    pub attractive_pheromone: KeyCode,
    /// Place a repulsive pheromone
    pub repulsive_pheromone: UserInput,
}

/// Add a default control scheme
impl Default for HiveMindControls {
    fn default() -> Self {
        Self {
            attractive_pheromone: KeyCode::Space,
            repulsive_pheromone: UserInput::chord([KeyCode::LShift, KeyCode::Space]),
        }
    }
}

/// Startup system initializing the [`HiveMind`].
fn initialize_hive_mind(mut commands: Commands) {
    let controls = HiveMindControls::default();
    commands
        .spawn_empty()
        .insert(HiveMind)
        .insert(InputManagerBundle::<HiveMindAction> {
            input_map: InputMap::new([
                (
                    controls.attractive_pheromone.into(),
                    HiveMindAction::PlaceAttractivePheromone,
                ),
                (
                    controls.repulsive_pheromone,
                    HiveMindAction::PlaceRepulsivePheromone,
                ),
            ]),
            ..default()
        })
        .insert(InputManagerBundle::<TileSelectionAction> {
            input_map: TileSelectionAction::default_input_map(),
            ..default()
        })
        .insert(SelectedTiles::default());
}

// TODO: figure out a different control scheme
/// Place pheromone, if the mouse is hovered over a hex tile.
fn place_pheromone(
    mut signal_create_evw: EventWriter<SignalModificationEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
    hive_mind_query: Query<&ActionState<HiveMindAction>, With<HiveMind>>,
) {
    let hive_mind_state = hive_mind_query.single();

    if hive_mind_state.pressed(HiveMindAction::PlaceAttractivePheromone) {
        if let Some(pos) = cursor_tile_pos.maybe_tile_pos() {
            signal_create_evw.send(SignalModificationEvent::SignalIncrement {
                emitter: Emitter::Stock(PheromoneAttract),
                pos,
                increment: 0.1,
            })
        }
    }
    // TODO: Fix the failing clamp in curves
    if hive_mind_state.pressed(HiveMindAction::PlaceRepulsivePheromone) {
        if let Some(pos) = cursor_tile_pos.maybe_tile_pos() {
            signal_create_evw.send(SignalModificationEvent::SignalIncrement {
                emitter: Emitter::Stock(PheromoneRepulse),
                pos,
                increment: 0.01,
            });
        }
    }
}
