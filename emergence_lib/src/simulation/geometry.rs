//! Manages the game world's grid and data tied to that grid

use bevy::{
    prelude::{Component, Deref, DerefMut, Entity, Resource},
    utils::HashMap,
};
use hexx::{Hex, HexLayout};

/// A hex-based coordinate, that represents exactly one tile.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct TilePos {
    /// The underlying hex coordinate
    pub hex: Hex,
}

/// The overall size and arrangement of the map.
#[derive(Debug, Resource)]
pub struct MapGeometry {
    /// The size and orientation of the map.
    pub layout: HexLayout,
    /// Which tile entity is stored at each tile position
    pub tiles_index: HashMap<TilePos, Entity>,
    /// Which structure is stored at each tile position
    pub structure_index: HashMap<TilePos, Entity>,
}

impl Default for MapGeometry {
    fn default() -> Self {
        MapGeometry {
            layout: HexLayout::default(),
            tiles_index: HashMap::default(),
            structure_index: HashMap::default(),
        }
    }
}
