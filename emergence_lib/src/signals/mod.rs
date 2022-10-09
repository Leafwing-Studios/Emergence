use bevy::prelude::*;

pub mod diffusion;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalId {
    Unspecified,
    Ant,
    Plant,
    Fungus,
}

impl Default for SignalId {
    fn default() -> Self {
        SignalId::Unspecified
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignalType {
    Passive(SignalId),
    Push(SignalId),
    Pull(SignalId),
    Work,
}

impl Default for SignalType {
    fn default() -> Self {
        Self::Passive(SignalId::Unspecified)
    }
}
