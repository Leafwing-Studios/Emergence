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
            .add_plugin(InputManagerPlugin::<DevAction>::default())
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
}

/// Enumerates the actions a developer can take.
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DevAction {
    /// Toggle the overall developer mode setting
    ToggleDevMode,
    // TODO: make debug labels
    /// Toggle tilemap labels tools
    ToggleTileLabels,
    /// Toggle rendering info
    ToggleInfoText,
    /// Toggle the inspector
    ToggleInspector,
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

// TODO: rework this to use LWIM conventions for mapping controls
/// Interface for developer controls
pub struct DevControls {
    /// Toggle the dev mode
    pub toggle_dev_mode: UserInput,
    /// Toggle the tile label
    pub toggle_tile_labels: UserInput,
    /// Toggle the fps ui
    pub toggle_fps: UserInput,
    /// Toggle the inspector
    pub toggle_inspector: UserInput,
}

/// Add default developer controls
impl Default for DevControls {
    fn default() -> Self {
        Self {
            toggle_dev_mode: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::D]),
            toggle_tile_labels: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::T]),
            toggle_fps: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::V]),
            toggle_inspector: UserInput::chord([KeyCode::LControl, KeyCode::LShift, KeyCode::I]),
        }
    }
}

/// Startup system initializing the [`HiveMind`].
fn initialize_hive_mind(mut commands: Commands) {
    let controls = HiveMindControls::default();
    let dev_controls = DevControls::default();
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
            ]),
        })
        .insert(InputManagerBundle::<DevAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (dev_controls.toggle_dev_mode, DevAction::ToggleDevMode),
                (dev_controls.toggle_tile_labels, DevAction::ToggleTileLabels),
                (dev_controls.toggle_fps, DevAction::ToggleInfoText),
                (dev_controls.toggle_inspector, DevAction::ToggleInspector),
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
    // TODO: Fix the failing clamp in curves
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
    dev: Query<&ActionState<DevAction>, With<HiveMind>>,
    mut debug_info: ResMut<DebugInfo>,
) {
    let dev = dev.single();
    let dev_mode = dev.just_pressed(DevAction::ToggleDevMode);
    let tile_labels = dev.just_pressed(DevAction::ToggleTileLabels);
    let fps_info = dev.just_pressed(DevAction::ToggleInfoText);
    let inspector = dev.just_pressed(DevAction::ToggleInspector);

    // Toggle the dev mode so that what happens is intuitive to the user
    if dev_mode {
        if debug_info.dev_mode {
            debug_info.disable();
            info!("Debug Info disabled");
        } else {
            debug_info.enable();
            info!("Debug Info enabled");
        }
    }

    // Toggle the tile labels, but also make sure that is makes sense to do so
    if tile_labels && debug_info.dev_mode {
        if debug_info.show_tile_labels {
            debug_info.show_tile_labels = false;
            info!("Tile Labels off");
        } else {
            debug_info.show_tile_labels = true;
            info!("Tile Labels on");
        }
    }

    // Toggle the FPS info
    if fps_info && debug_info.dev_mode {
        if debug_info.show_fps_info {
            debug_info.show_fps_info = false;
            info!("FPS info off");
        } else {
            debug_info.show_fps_info = true;
            info!("FPS info on");
        }
    }

    // Toggle the inspector
    if inspector && debug_info.dev_mode {
        if debug_info.show_inspector {
            debug_info.show_inspector = false;
            info!("Egui inspector off");
        } else {
            debug_info.show_inspector = true;
            info!("Egui inspector on");
        }
    }
}
