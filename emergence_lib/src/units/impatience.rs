use bevy::prelude::*;
use core::fmt::Display;

/// The patience of a unit.
///
/// If current >= max, they will abandon their current goal.
#[derive(Debug, Clone, PartialEq, Component, Resource)]
pub(crate) struct ImpatiencePool {
    /// The current impatience of this unit.
    current: u8,
    /// The maximum impatience of this unit.
    max: u8,
}

impl ImpatiencePool {
    /// Creates a new impatience pool with the provided `max` value.
    pub(super) fn new(max: u8) -> Self {
        ImpatiencePool { current: 0, max }
    }

    /// Is this unit out of patience?
    pub(super) fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Increase the current impatience by 1
    pub(super) fn increment(&mut self) {
        self.current += 1;
    }

    /// Resets the current impatience to 0
    pub(super) fn reset(&mut self) {
        self.current = 0;
    }
}

impl Display for ImpatiencePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current, self.max)
    }
}
