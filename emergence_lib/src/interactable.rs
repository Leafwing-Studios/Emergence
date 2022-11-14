//! Exhaustively enumerates all of the types of objects that can be interacted with.
//!
//! Used for signalling, unit behaviors and more.

use bevy::ecs::component::Component;

/// An object that can be interacted with by units
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Interactable {
    /// Plant
    Plant,
    /// Fungus
    Fungus,
    /// Ant
    Ant,
}
