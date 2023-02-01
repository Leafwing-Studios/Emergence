//! Manages the game world's grid and data tied to that grid

use bevy::prelude::{Component, Deref, DerefMut};
use hexx::Hex;

/// A hex-based coordinate, that represents exactly one tile.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct TilePos {
    /// The underlying hex coordinate
    pub hex: Hex,
}
