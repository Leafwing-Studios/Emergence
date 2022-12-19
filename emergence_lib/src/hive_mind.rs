//! Represents the player.

use crate::cursor::CursorTilePos;
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::{PheromoneAttract, PheromoneRepulse};
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use debug_tools::{DebugInfo, DebugToolsPlugin};
use leafwing_input_manager::prelude::*;

/// Provides the interface between the player and the hive.
pub struct HiveMindPlugin;

impl Plugin for HiveMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<HiveMindAction>::default())
            .init_resource::<DebugInfo>()
            .add_plugin(DebugToolsPlugin)
            .add_startup_system(initialize_hive_mind)
            .add_system(place_pheromone)
            .add_system(show_debug_info);
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
    /// Toggle tilemap labels tools
    ShowDebugLabels,
    /// Toggle rendering info
    ShowInfoText,
}

// This needs to be converted to LWIM from just inputs
/// Interface for player controls
#[derive(Resource)]
pub struct HiveMindControls {
    /// Place an attractive pheromone
    pub attractive_pheromone: KeyCode,
    /// Place a repulsive pheromone
    pub repulsive_pheromone: UserInput,
    /// Toggle the fps window
    pub toggle_fps: KeyCode,
}

impl Default for HiveMindControls {
    fn default() -> Self {
        Self {
            attractive_pheromone: KeyCode::Space,
            repulsive_pheromone: UserInput::chord([KeyCode::LShift, KeyCode::Space]),
            toggle_fps: KeyCode::V,
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
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (
                    controls.attractive_pheromone.into(),
                    HiveMindAction::PlaceAttractivePheromone,
                ),
                (
                    controls.repulsive_pheromone,
                    HiveMindAction::PlaceRepulsivePheromone,
                ),
                (controls.toggle_fps.into(), HiveMindAction::ShowInfoText),
            ]),
        });
}

// TODO: figure out a different control scheme
/// Place pheromone, if the mouse is hovered over a hex tile.
fn place_pheromone(
    mut signal_create_evw: EventWriter<SignalModificationEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
    hive_mind_query: Query<&ActionState<HiveMindAction>, With<HiveMind>>,
) {
    let hive_mind_state = hive_mind_query.single();

    if hive_mind_state.pressed(HiveMindAction::PlaceAttractivePheromone)
        && (*cursor_tile_pos).is_some()
    {
        signal_create_evw.send(SignalModificationEvent::SignalIncrement {
            emitter: Emitter::Stock(PheromoneAttract),
            pos: (*cursor_tile_pos).unwrap(),
            increment: 0.1,
        })
    }
    if hive_mind_state.pressed(HiveMindAction::PlaceRepulsivePheromone)
        && (*cursor_tile_pos).is_some()
    {
        signal_create_evw.send(SignalModificationEvent::SignalIncrement {
            emitter: Emitter::Stock(PheromoneRepulse),
            pos: (*cursor_tile_pos).unwrap(),
            increment: 0.01,
        });
    }
}

/// Toggle showing debug info   
fn show_debug_info(
    hive_mind: Query<&ActionState<HiveMindAction>, With<HiveMind>>,
    mut bools: ResMut<DebugInfo>,
) {
    let hive_mind = hive_mind.single();
    let fps_info = hive_mind.pressed(HiveMindAction::ShowInfoText);

    if fps_info && bools.show_fps_info {
        bools.show_fps_info = false;
    } else if fps_info && !bools.show_fps_info {
        bools.show_fps_info = true;
    }
}
