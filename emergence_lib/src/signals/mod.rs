//! Models signals emitted by the hive mind, or units of the hive.
//!
//! Signals diffuse, can be convected, and so on.
pub mod configs;
pub mod emitters;
pub mod tile_signals;
use crate::enum_iter::IterableEnum;
use crate::signals::configs::{SignalConfig, SignalConfigs};
use crate::signals::emitters::Emitter;
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::hex_patch::HexPatchLocation;
use crate::simulation::map::index::MapIndex;
use crate::simulation::map::MapPositions;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;

/// This plugin manages all aspects of signals:
/// * creation,
/// * diffusion, advection, reaction
/// * presenting map overlays
pub struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SignalConfigs>()
            .add_event::<SignalModificationEvent>()
            .add_startup_system_to_stage(StartupStage::PostStartup, initialize_map_signals)
            .add_system(handle_signal_modification_events)
            .add_system(decay.after(handle_signal_modification_events))
            .add_system(compute_deltas.after(decay))
            .add_system(apply_deltas.after(compute_deltas));
    }
}

/// Initialize [`TileSignals`] for each position.
///
/// This is a startup system that should run after terrain generation, i.e. in
/// [`StartupStage::PostStartup`]. It will panic if it cannot find the [`MapPositions`] resource.
fn initialize_map_signals(mut commands: Commands, map_positions: Res<MapPositions>) {
    commands.insert_resource(MapIndex::<TileSignals>::default_from_template(
        &map_positions,
    ))
}

/// Event modifying a signal at a tile.
pub enum SignalModificationEvent {
    /// Increment/decrement a signal by requested amount.
    SignalIncrement {
        /// Emitter id of the signal.
        emitter: Emitter,
        /// Tile position of the signal.
        pos: TilePos,
        /// Amount to be incremented/decremented by.
        increment: f32,
    },
    /// Create a signal at a tile, initialized with the given settings.
    SignalCreate {
        /// Emitter id of the signal.
        emitter: Emitter,
        /// Tile position of the signal.
        pos: TilePos,
        /// Initial signal value.
        initial: Signal,
        /// Configuration of the signal.
        config: SignalConfig,
    },
}

/// An event that requests incrementing of a signal at a given tile position.
pub struct SignalIncrementEvent {}

/// Reads [`SignalIncrementEvent`]s to create new signals on the map.
fn handle_signal_modification_events(
    mut modification_events: EventReader<SignalModificationEvent>,
    map_signals: Res<MapIndex<TileSignals>>,
    mut signal_configs: ResMut<SignalConfigs>,
) {
    for creation_event in modification_events.iter() {
        match creation_event {
            SignalModificationEvent::SignalIncrement {
                emitter,
                pos,
                increment,
            } => {
                map_signals
                    .get(pos)
                    .unwrap()
                    .get_mut()
                    .increment(emitter, *increment);
            }
            SignalModificationEvent::SignalCreate {
                emitter,
                pos,
                initial,
                config,
            } => {
                signal_configs.insert(*emitter, *config);
                map_signals
                    .get(pos)
                    .unwrap()
                    .get_mut()
                    .insert(*emitter, *initial);
            }
        }
    }
}

/// System that decays signals at all positions, at their configured per-tick decay probability
fn decay(mut map_signals: ResMut<MapIndex<TileSignals>>, signal_configs: Res<SignalConfigs>) {
    for tile_signals in map_signals.values_mut() {
        tile_signals.get_mut().decay(&signal_configs);
    }
}

/// Compute changes (deltas) in signal values at tiles, due to movement of signal between
/// tiles.
///
/// Currently movement only occurs due to diffusion.
fn compute_deltas(
    map_positions: Res<MapPositions>,
    mut map_signals: ResMut<MapIndex<TileSignals>>,
    signal_configs: Res<SignalConfigs>,
) {
    for tile_pos in map_positions.iter_positions() {
        let current_values = map_signals.get(tile_pos).unwrap().read().current_values();
        let signals_patch = map_signals.get_patch_mut(tile_pos).unwrap();

        for (emitter_id, current_value) in current_values {
            if let Some(signal_config) = signal_configs.get(&emitter_id) {
                // TODO: this should also be cached in a MapIndex?
                // normalize the diffusion factors into a probability
                let neighbor_diffusion_probability = if signal_config.diffusion_factor > 0.0 {
                    let count = map_positions.get_patch_count(tile_pos).unwrap();
                    signal_config.diffusion_factor / (signal_config.diffusion_factor * count as f32)
                } else {
                    0.0
                };

                let mut total_outgoing = 0.0;
                for location in HexPatchLocation::variants() {
                    if let Some(s) = signals_patch.get_inner_mut(location) {
                        let delta = neighbor_diffusion_probability * current_value;
                        s.get_mut().increment_incoming(&emitter_id, delta);
                        total_outgoing += delta;
                    }
                }
                signals_patch
                    .get_inner_mut(HexPatchLocation::Center)
                    .unwrap()
                    .get_mut()
                    .increment_outgoing(&emitter_id, total_outgoing);
            } else {
                error!("No config found for {emitter_id:?}!");
            }
        }
    }
}

/// Applies deltas due to movement of signals between tiles.
///
/// Should run after [`compute_deltas`].
fn apply_deltas(mut map_signals: ResMut<MapIndex<TileSignals>>) {
    for tile_signals in map_signals.values_mut() {
        tile_signals.get_mut().apply_deltas();
    }
}

/// A diffusible signal at a given tile.
#[derive(Default, Debug, Clone, Copy)]
pub struct Signal {
    /// The value of the signal at this tick.
    current_value: f32,
    /// The amount of signal that will be coming into this tile this tick.
    ///
    /// Generally, this will be based on [`current_value`](Signal::current_value) of neighboring tiles.
    incoming: f32,
    /// The amount of signal that will be leaving this tile this tick.
    ///
    /// Generally, this will be based on [`current_value`](Signal::current_value).
    outgoing: f32,
}

impl Signal {
    /// Create new [`Signal`] with given starting `value` and `color`.
    pub fn new(value: f32) -> Signal {
        Signal {
            current_value: value,
            incoming: 0.0,
            outgoing: 0.0,
        }
    }

    /// Apply accumulated `incoming`/`outgoing` to the `current_value`, while ensuring that the
    /// signal's value does not go below `0.0`.
    ///
    /// `incoming` and `outgoing` are reset to `0.0` once applied.
    fn apply_deltas(&mut self) {
        self.current_value = (self.current_value + self.incoming - self.outgoing).max(0.0);
        self.incoming = 0.0;
        self.outgoing = 0.0;
    }
}

/// Information carried by the signal, which is typically translated into an activity instruction.
#[derive(Debug, Clone, Copy)]
pub enum SignalInfo {
    /// Signal that does not carry an instruction.
    Passive(Emitter),
    /// Signal with a push (drop-off) instruction.
    Push(Emitter),
    /// Signal with a pull (fetch) instruction.
    Pull(Emitter),
    /// Signal that requests work be carried out.
    Work,
}
