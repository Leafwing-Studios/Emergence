//! Manages the game world's grid and data tied to that grid

use bevy::{
    prelude::{Component, Deref, DerefMut, Entity, Resource},
    utils::HashMap,
};
use hexx::{Direction, Hex, HexLayout};

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
    /// The number of tiles from the center to the edge of the map.
    ///
    /// Note that the central tile is not counted.
    pub radius: u32,
    /// Which tile entity is stored at each tile position
    pub tiles_index: HashMap<TilePos, Entity>,
    /// Which structure is stored at each tile position
    pub structure_index: HashMap<TilePos, Entity>,
}

impl Default for MapGeometry {
    fn default() -> Self {
        MapGeometry {
            layout: HexLayout::default(),
            radius: 50,
            tiles_index: HashMap::default(),
            structure_index: HashMap::default(),
        }
    }
}

/// Rotates a hex [`Direction`] one step clockwise.
#[must_use]
pub fn clockwise(direction: Direction) -> Direction {
    use Direction::*;
    match direction {
        BottomRight => Bottom,
        TopRight => BottomRight,
        Top => TopRight,
        TopLeft => Top,
        BottomLeft => TopLeft,
        Bottom => BottomLeft,
    }
}

/// Rotates a hex [`Direction`] one step counterclockwise.
#[must_use]
pub fn counterclockwise(direction: Direction) -> Direction {
    use Direction::*;
    match direction {
        BottomRight => TopRight,
        TopRight => Top,
        Top => TopLeft,
        TopLeft => BottomLeft,
        BottomLeft => Bottom,
        Bottom => BottomRight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_reverse_each_other() {
        for direction in Direction::ALL_DIRECTIONS {
            assert_eq!(direction, counterclockwise(clockwise(direction)));
            assert_eq!(direction, clockwise(counterclockwise(direction)));
        }
    }

    #[test]
    fn six_rotations_comes_home() {
        for direction in Direction::ALL_DIRECTIONS {
            let mut cw_direction = direction;
            let mut ccw_direction = direction;

            for _ in 0..6 {
                cw_direction = clockwise(cw_direction);
                ccw_direction = counterclockwise(ccw_direction);
            }

            assert_eq!(direction, cw_direction);
            assert_eq!(direction, ccw_direction);
        }
    }
}
