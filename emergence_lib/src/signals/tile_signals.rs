use crate::signals::configs::SignalConfigs;
use crate::signals::emitters::Emitter;
use crate::signals::Signal;
use bevy::prelude::*;
use dashmap::DashMap;

/// Keeps track of the different signals present at a tile.
///
/// Internally it is a [`DashMap`] with keys of type [`Emitter`] and values of type [`Signal`].
///
/// It provides various public interfaces to interact with signals.
#[derive(Component, Default, Debug)]
pub struct TileSignals {
    map: DashMap<Emitter, Signal>,
}

impl TileSignals {
    /// Increment the change in signal due to signal leaving this tile.
    pub fn emitters(&self) -> Vec<Emitter> {
        self.map.iter().map(|kv| *kv.key()).collect()
    }

    /// Get the current values of the signals at this tile.
    pub fn current_values(&self) -> Vec<(Emitter, f32)> {
        self.map
            .iter()
            .map(|kv| (*kv.key(), kv.value().current_value))
            .collect()
    }

    pub fn insert(&mut self, emitter: Emitter, signal: Signal) {
        self.map.insert(emitter, signal);
    }

    /// Increment the change in signal due to signal entering this tile.
    ///
    /// If there is no signal with the specified `Emitter`, a new one will be initialized.
    ///
    /// This change will be applied before the next tick, but after all diffusion has been done.
    pub fn increment_incoming(&mut self, emitter: &Emitter, delta: f32) {
        if let Some(signal) = self.map.get_mut(emitter) {
            signal.incoming += delta;
        } else {
            let mut new_signal = Signal::new(0.0);
            new_signal.incoming = delta;
            self.map.insert(*emitter, new_signal);
        }
    }

    /// Increment the change in signal due to signal leaving this tile.
    ///
    /// Panics if there is no signal from the specified `Emitter`.
    ///
    /// This change will be applied before the next tick, but after all diffusion has been done.
    pub fn increment_outgoing(&mut self, emitter: &Emitter, delta: f32) {
        let signal = self.map.get_mut(emitter).unwrap();
        signal.outgoing += delta;
    }

    /// Decay signal at the tile.
    ///
    /// Panics if there is no signal from the specified `Emitter`.
    pub fn decay(&mut self, signal_configs: &SignalConfigs) {
        for mut emitter_signal in self.map.iter_mut() {
            let (emitter, signal) = emitter_signal.pair_mut();
            let config = signal_configs.get(emitter).unwrap();
            signal.current_value = signal.current_value * config.decay_probability;
        }
    }

    /// Apply accumulated `incoming`/`outgoing` to the `current_value` for each signal at this tile.
    pub fn apply_deltas(&mut self) {
        for mut emitter_signal in self.map.iter_mut() {
            let signal = emitter_signal.value_mut();
            signal.apply_deltas();
        }
    }
}
