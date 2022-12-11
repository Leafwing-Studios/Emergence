//! Represents the player.

use crate::cursor::CursorTilePos;
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::{PheromoneAttract, PheromoneRepulse};
use crate::signals::SignalModificationEvent;
use bevy::prelude::*;
use debug_tools::debug_ui::FpsText;
use debug_tools::*;
use leafwing_input_manager::prelude::*;

/// Provides the interface between the player and the hive.
pub struct HiveMindPlugin;

impl Plugin for HiveMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<HiveMindAction>::default())
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

/// Startup system initializing the [`HiveMind`].
fn initialize_hive_mind(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(HiveMind)
        .insert(InputManagerBundle::<HiveMindAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (
                    KeyCode::Space.into(),
                    HiveMindAction::PlaceAttractivePheromone,
                ),
                (
                    UserInput::chord([KeyCode::LShift, KeyCode::Space]),
                    HiveMindAction::PlaceRepulsivePheromone,
                ),
                (KeyCode::D.into(), HiveMindAction::ShowDebugLabels),
                (KeyCode::V.into(), HiveMindAction::ShowInfoText),
            ]),
        });
}

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
        info!("replusing");
    }
}

fn show_debug_info(
    hive_mind: Query<&ActionState<HiveMindAction>, With<HiveMind>>,
    bools: Query<&DebugInfo, With<FpsText>>,
) {
    let hive_mind = hive_mind.single();
    let mut bools = *bools.single();
    let tile_labels = hive_mind.pressed(HiveMindAction::ShowDebugLabels);
    let fps_info = hive_mind.pressed(HiveMindAction::ShowInfoText);

    if tile_labels && bools.show_tile_label {
        info!("previous show label is {:?}", bools.show_tile_label);
        bools.show_tile_label = false;
        info!("show label is {:?}", bools.show_tile_label)
    } else if tile_labels && !bools.show_tile_label {
        bools.show_tile_label = true;
    }
    if fps_info && bools.show_fps_info {
        info!("previous show fps {:?} ", bools.show_fps_info);
        bools.show_fps_info = false;
        info!("show fps is {:?}", bools.show_fps_info)
    } else if fps_info && !bools.show_fps_info {
        bools.show_fps_info = true;
    }
}
