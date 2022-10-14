pub mod configs;
pub mod emitters;
pub mod map_overlay;
pub mod tile_signals;

use crate::curves::ergonomic_sigmoid;
use crate::signals::configs::{SignalColorConfig, SignalConfig, SignalConfigs};
use crate::signals::emitters::Emitter;
use crate::signals::map_overlay::MapOverlayPlugin;
use crate::signals::tile_signals::TileSignals;
use crate::terrain::generation::TerrainTilemap;
use crate::tiles::position::HexNeighbors;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ecs_tilemap::tiles::TileStorage;

/// This plugin manages all aspects of signals:
/// * creation,
/// * diffusion, advection, reaction
/// * presenting map overlays
pub struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SignalConfigs>()
            .add_event::<SignalCreateEvent>()
            .add_plugin(MapOverlayPlugin)
            .add_startup_system_to_stage(StartupStage::PostStartup, initialize_tile_signals)
            .add_system(create)
            .add_system(decay.after(create))
            .add_system(compute_deltas.after(decay))
            .add_system(apply_deltas.after(compute_deltas));
    }
}

/// Initialize [`TileSignals`] for each terrain tile.
///
/// This is a startup system that should run after terrain generation, i.e. in
/// [`StartupStage::PostStartup`]. It will panic if it cannot find the tile storage for the terrain
/// tile map.
fn initialize_tile_signals(
    mut commands: Commands,
    terrain_tile_storage_query: Query<&TileStorage, With<TerrainTilemap>>,
) {
    let terrain_tile_storage = terrain_tile_storage_query.single();
    let tile_signals = terrain_tile_storage
        .iter()
        .flatten()
        .map(|e| (*e, (TileSignals::default(),)))
        .collect::<Vec<(Entity, (TileSignals,))>>();
    commands.insert_or_spawn_batch(tile_signals);
}

/// An event that requests initialization of a new signal at a given tile position.
pub struct SignalCreateEvent {
    pub emitter: Emitter,
    pub pos: TilePos,
    pub initial: Signal,
    pub config: SignalConfig,
}

/// Reads [`SignalCreateEvent`]s to create new signals on the map.
fn create(
    mut creation_events: EventReader<SignalCreateEvent>,
    mut signals_query: Query<&mut TileSignals>,
    terrain_tile_storage_query: Query<&TileStorage, With<TerrainTilemap>>,
    mut signal_configs: ResMut<SignalConfigs>,
) {
    let terrain_tile_storage = terrain_tile_storage_query.single();
    for creation_event in creation_events.iter() {
        let SignalCreateEvent {
            emitter,
            pos,
            initial,
            config,
        } = creation_event;
        if let Some(tile_entity) = terrain_tile_storage.checked_get(pos) {
            let mut tile_signals = signals_query.get_mut(tile_entity).unwrap();

            signal_configs.insert(*emitter, *config);
            tile_signals.insert(*emitter, *initial);
        }
    }
}

/// System that decays signals at all tiles, at their configured per-tick decay probability
fn decay(mut signals_query: Query<&mut TileSignals>, signal_configs: Res<SignalConfigs>) {
    for mut tile_signals in signals_query.iter_mut() {
        tile_signals.decay(&signal_configs)
    }
}

/// Compute changes (deltas) in signal values at tiles, due to movement of signal between
/// tiles.
///
/// Currently movement only occurs due to diffusion.
fn compute_deltas(
    terrain_tilemap_query: Query<(&TilemapSize, &TileStorage), With<TerrainTilemap>>,
    signals_query: Query<(&TilePos, &mut TileSignals)>,
    signal_configs: Res<SignalConfigs>,
) {
    let (map_size, tile_storage) = terrain_tilemap_query.single();
    for (tile_pos, signals) in signals_query.iter() {
        for (emitter_id, current_value) in signals.current_values() {
            let signal_config = signal_configs.get(&emitter_id).unwrap();
            // FIXME: these neighbor positions/entities should be cached (in a resource?)
            let neighbor_entities =
                HexNeighbors::get_neighbors(tile_pos, map_size).entities(tile_storage);
            let mutable_neighbor_signals =
                neighbor_entities.and_then(|entity| signals_query.get(entity).map(|(_, s)| s).ok());
            // normalize the diffusion factors into a probability
            let neighbor_diffusion_probability = if signal_config.diffusion_factor > 0.0 {
                signal_config.diffusion_factor
                    / (signal_config.diffusion_factor
                        * ((mutable_neighbor_signals.iter().count() + 1) as f32))
            } else {
                0.0
            };
            let mut total_outgoing = 0.0;
            for s in mutable_neighbor_signals.iter() {
                let delta = neighbor_diffusion_probability * current_value;
                s.increment_incoming(&emitter_id, delta);
                total_outgoing += delta;
            }
            signals.increment_outgoing(&emitter_id, total_outgoing);
        }
    }
}

/// Applies deltas due to movement of signals between tiles.
///
/// Should run after [`compute_deltas`].
fn apply_deltas(mut signals_query: Query<&mut TileSignals>) {
    for mut tile_signals in signals_query.iter_mut() {
        tile_signals.apply_deltas()
    }
}

/// A diffusible signal.
#[derive(Default, Debug, Clone, Copy)]
pub struct Signal {
    current_value: f32,
    incoming: f32,
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
    /// minimum accumulated value is `0.0`.
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
            color.set_a(ergonomic_sigmoid(
                self.current_value,
                0.0,
                1.0,
                color_config.zero_value,
                color_config.one_value,
            ));

            color
        })
    }
}

/// Information carried by the signal, which is typically translated into an activity instruction.
#[derive(Debug, Clone, Copy)]
pub enum SignalInfo {
    Passive(Emitter),
    Push(Emitter),
    Pull(Emitter),
    Work,
}

impl Default for SignalInfo {
    fn default() -> Self {
        Self::Passive(Emitter::default())
    }
}
