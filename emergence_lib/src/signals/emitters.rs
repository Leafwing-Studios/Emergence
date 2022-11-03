//! Data for entities which can emit a signal.

use bevy::prelude::*;
use emergence_macros::IterableEnum;

/// All signal emitters have an `EmitterId`, which is essentially a `u16`.
#[derive(Component, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Emitter {
    /// A custom signal, designed by the player.
    Custom(u16),
    /// A stock signal, which comes pre-defined by the game.
    Stock(StockEmitter),
}

impl Default for Emitter {
    fn default() -> Self {
        Self::Stock(StockEmitter::Unspecified)
    }
}

use crate as emergence_lib;
/// Enumerates stock signal emitters.
#[derive(Debug, Default, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, IterableEnum)]
pub enum StockEmitter {
    /// Emitter is unspecified.
    #[default]
    Unspecified,
    /// Emitter is stock ant.
    Ant,
    /// Emitter is stock fungus.
    Fungus,
    /// Hive mind's signal for attracting ants.
    PheromoneAttract,
    /// Hive mind's signal for repelling ants.
    PheromoneRepulse,
    /// Emitter is a plant.
    Plant,
}
