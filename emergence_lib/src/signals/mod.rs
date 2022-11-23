//! Models signals emitted by the hive mind, or units of the hive.
//!
//! Signals diffuse, can be convected, and so on.
pub mod configs;
pub mod emitters;
pub mod map_overlay;
pub mod tile_signals;
use crate::curves::Mapping;
use crate::signals::configs::{SignalColorConfig, SignalConfig, SignalConfigs};
use crate::signals::emitters::Emitter;
use crate::signals::map_overlay::MapOverlayPlugin;
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::resources::MapResource;
use crate::simulation::map::MapPositions;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HEX_DIRECTIONS;
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
            .add_plugin(MapOverlayPlugin)
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
    commands.insert_resource(MapResource::<TileSignals>::default_from_template(
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
    map_signals: Res<MapResource<TileSignals>>,
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
fn decay(mut map_signals: ResMut<MapResource<TileSignals>>, signal_configs: Res<SignalConfigs>) {
    for tile_signals in map_signals.values_mut() {
        tile_signals.get_mut().decay(&signal_configs);
    }
}

/// Compute changes (deltas) in signal values at graphics, due to movement of signal between
/// graphics.
///
/// Currently movement only occurs due to diffusion.
fn compute_deltas(
    map_positions: Res<MapPositions>,
    mut map_signals: ResMut<MapResource<TileSignals>>,
    signal_configs: Res<SignalConfigs>,
) {
    for tile_pos in map_positions.iter_positions() {
        let current_values = map_signals.get(tile_pos).unwrap().read().current_values();
        let neighbors = map_signals.get_neighbors_mut(tile_pos).unwrap();

        for (emitter_id, current_value) in current_values {
            let signal_config = signal_configs.get(&emitter_id).unwrap();

            // TODO: this should also be cached in a MapResource?
            // normalize the diffusion factors into a probability
            let neighbor_diffusion_probability = if signal_config.diffusion_factor > 0.0 {
                let count = map_positions.get_neighbor_count(tile_pos).unwrap();
                signal_config.diffusion_factor
                    / (signal_config.diffusion_factor * (count + 1) as f32)
            } else {
                0.0
            };

            let mut total_outgoing = 0.0;
            for direction in HEX_DIRECTIONS.into_iter() {
                if let Some(s) = neighbors.get_mut(direction.into()) {
                    let delta = neighbor_diffusion_probability * current_value;
                    s.get_mut().increment_incoming(&emitter_id, delta);
                    total_outgoing += delta;
                }
            }
            map_signals
                .get_mut(tile_pos)
                .unwrap()
                .get_mut()
                .increment_outgoing(&emitter_id, total_outgoing);
        }
    }
}

/// Applies deltas due to movement of signals between graphics.
///
/// Should run after [`compute_deltas`].
fn apply_deltas(mut map_signals: ResMut<MapResource<TileSignals>>) {
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
    /// Generally, this will be based on [`current_value`](Signal::current_value) of neighboring graphics.
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

    /// Compute the color of this signal given a color configuration, if the signal's color
    /// configuration indicates it is to be visible.
    fn compute_color(&self, color_config: &SignalColorConfig) -> Option<Color> {
        color_config.is_visible.then_some({
            // This produces a Color::Rgba variant.
            let mut color = Color::from(color_config.rgb_color);

            // What are the possible values for a signal? [0, \infty)
            // What are we mapping to? [0, 1), the alpha component of our color
            // Use a shifted sigmoid to represent this
            color.set_a(color_config.sigmoid.map(self.current_value));

            color
        })
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

impl Default for SignalInfo {
    fn default() -> Self {
        Self::Passive(Emitter::default())
    }
}
