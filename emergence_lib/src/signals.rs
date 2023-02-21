//! Signals are used for pathfinding and decision-making.
//!
//! By collecting information about the local environment into a slowly updated, tile-centric data structure,
//! we can scale path-finding and decisionmaking in a clear and comprehensible way.

use bevy::{prelude::*, utils::HashMap};

use crate::{
    items::ItemId,
    simulation::geometry::{MapGeometry, TilePos},
    structures::StructureId,
};

/// The resources and systems need to work with signals
pub(crate) struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Signals>();
    }
}

/// The central resource that tracks all signals.
#[derive(Resource, Debug, Default)]
pub(crate) struct Signals {
    /// The spatialized map for each signal
    maps: HashMap<SignalType, SignalMap>,
}

impl Signals {
    /// Returns the signal strength of `signal_type` at the given `tile_pos`.
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, signal_type: SignalType, tile_pos: TilePos) -> SignalStrength {
        match self.maps.get(&signal_type) {
            Some(map) => map.get(tile_pos),
            None => SignalStrength::ZERO,
        }
    }

    /// Returns the complete set of signals at the given `tile_pos`.
    ///
    /// This is useful for decision-making.
    fn all_signals_at_position(&self, tile_pos: TilePos) -> HashMap<SignalType, SignalStrength> {
        let mut all_signals = HashMap::new();
        for &signal_type in self.maps.keys() {
            let strength = self.get(signal_type, tile_pos);
            all_signals.insert(signal_type, strength);
        }

        all_signals
    }

    /// Returns the signal strength of the type `signal_type` in `tile_pos` and its 6 surrounding neighbors.
    fn neighboring_signals(
        &self,
        signal_type: SignalType,
        tile_pos: TilePos,
        map_geometry: &MapGeometry,
    ) -> HashMap<TilePos, SignalStrength> {
        let mut signal_strength_map = HashMap::with_capacity(7);

        signal_strength_map.insert(tile_pos, self.get(signal_type, tile_pos));
        for neighbor in tile_pos.neighbors(map_geometry) {
            signal_strength_map.insert(neighbor, self.get(signal_type, neighbor));
        }

        signal_strength_map
    }
}

/// Stores the [`SignalStrength`] of the given [`SignalType`] at each [`TilePos`].
#[derive(Debug, Default)]
struct SignalMap {
    /// The lookup data structure
    map: HashMap<TilePos, SignalStrength>,
}

impl SignalMap {
    /// Returns the signal strenth at the given [`TilePos`].
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, tile_pos: TilePos) -> SignalStrength {
        *self.map.get(&tile_pos).unwrap_or(&SignalStrength::ZERO)
    }
}

/// The variety of signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SignalType {
    /// Take this item away from here.
    Push(ItemId),
    /// Bring me an item of this type.
    Pull(ItemId),
    /// Has an item of this type, in case you were looking.
    Contains(StructureId),
    /// Perform work at this type of structure.
    Work(StructureId),
}

/// How strong a signal is.
///
/// This has a minimum value of 0.
#[derive(Debug, Default, Clone, Copy)]
struct SignalStrength(f32);

impl SignalStrength {
    /// No signal is present.
    const ZERO: SignalStrength = SignalStrength(0.);
}
