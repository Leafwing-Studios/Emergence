use bevy::prelude::*;

/// All signal emitters have an `EmitterId`, which is essentially a `u16`.
#[derive(Component, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Emitter {
    Custom(u16),
    Stock(StockEmitter),
}

impl Default for Emitter {
    fn default() -> Self {
        Self::Stock(StockEmitter::Unspecified)
    }
}

/// Enumerates stock signal emitters.
#[derive(Debug, Default, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum StockEmitter {
    #[default]
    Unspecified,
    Ant,
    Fungus,
    PheromoneAttract,
    PheromoneRepulse,
    Plant,
}
