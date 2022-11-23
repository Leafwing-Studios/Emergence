//! Code for managing data tied to (spatial) neighbors of a given position

use crate::simulation::map::resources::MapData;
use crate::simulation::map::MapGeometry;
use bevy::prelude::Res;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::{HexRowDirection, HEX_DIRECTIONS};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Debug;

/// Stores some data `T` associated with each neighboring hex cell, if present.
#[derive(Debug, Default)]
pub struct HexNeighbors<T> {
    /// The northern neighbor.
    north: Option<T>,
    /// The north-western neighbor.
    north_west: Option<T>,
    /// The south-western neighbor.
    south_west: Option<T>,
    /// The southern neighbor.
    south: Option<T>,
    /// The south-eastern neighbor.
    south_east: Option<T>,
    /// The north-eastern neighbor.
    north_east: Option<T>,
}

impl<T> HexNeighbors<T> {
    /// Get the neighbor in the specified direction.
    pub fn get(&self, direction: HexRowDirection) -> Option<&T> {
        match direction {
            HexRowDirection::North => self.north.as_ref(),
            HexRowDirection::NorthWest => self.north_west.as_ref(),
            HexRowDirection::SouthWest => self.south_west.as_ref(),
            HexRowDirection::South => self.south.as_ref(),
            HexRowDirection::SouthEast => self.south_east.as_ref(),
            HexRowDirection::NorthEast => self.north_east.as_ref(),
        }
    }

    /// Get a mutable reference to the neighbor in the specified direction.
    pub fn get_mut(&mut self, direction: HexRowDirection) -> Option<&mut T> {
        match direction {
            HexRowDirection::North => self.north.as_mut(),
            HexRowDirection::NorthWest => self.north_west.as_mut(),
            HexRowDirection::SouthWest => self.south_west.as_mut(),
            HexRowDirection::South => self.south.as_mut(),
            HexRowDirection::SouthEast => self.south_east.as_mut(),
            HexRowDirection::NorthEast => self.north_east.as_mut(),
        }
    }

    /// Set the data associated with the neighbor in the specified direction.
    pub fn set(&mut self, direction: HexRowDirection, data: T) {
        match direction {
            HexRowDirection::North => self.north.replace(data),
            HexRowDirection::NorthWest => self.north_west.replace(data),
            HexRowDirection::SouthWest => self.south_west.replace(data),
            HexRowDirection::South => self.south.replace(data),
            HexRowDirection::SouthEast => self.south_east.replace(data),
            HexRowDirection::NorthEast => self.north_east.replace(data),
        };
    }

    /// Iterate through existing neighbors.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        HEX_DIRECTIONS
            .into_iter()
            .filter_map(|direction| self.get(direction.into()))
    }

    /// Count the number of neighbors with [`Some`] data
    pub fn count(&self) -> usize {
        HEX_DIRECTIONS
            .into_iter()
            .filter_map(|direction| self.get(direction.into()))
            .count()
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by value.
    pub fn and_then<U, F>(self, f: F) -> HexNeighbors<U>
    where
        F: Fn(T) -> Option<U>,
    {
        HexNeighbors {
            north: self.north.and_then(&f),
            north_west: self.north_west.and_then(&f),
            south_west: self.south_west.and_then(&f),
            south: self.south.and_then(&f),
            south_east: self.south_east.and_then(&f),
            north_east: self.north_east.and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn and_then_ref<'a, U, F>(&'a self, f: F) -> HexNeighbors<U>
    where
        F: Fn(&'a T) -> Option<U>,
    {
        HexNeighbors {
            north: self.north.as_ref().and_then(&f),
            north_west: self.north_west.as_ref().and_then(&f),
            south_west: self.south_west.as_ref().and_then(&f),
            south: self.south.as_ref().and_then(&f),
            south_east: self.south_east.as_ref().and_then(&f),
            north_east: self.north_east.as_ref().and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with a [`map`](std::option::Option::map) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn map_ref<'a, U, F>(&'a self, f: F) -> HexNeighbors<U>
    where
        F: Fn(&'a T) -> U,
    {
        HexNeighbors {
            north: self.north.as_ref().map(&f),
            north_west: self.north_west.as_ref().map(&f),
            south_west: self.south_west.as_ref().map(&f),
            south: self.south.as_ref().map(&f),
            south_east: self.south_east.as_ref().map(&f),
            north_east: self.north_east.as_ref().map(&f),
        }
    }

    /// Generates `HexNeighbors<T>` from a closure that takes a hex direction, and outputs
    /// `Option<T>`.
    pub fn from_directional_closure<F>(f: F) -> HexNeighbors<T>
    where
        F: Fn(HexRowDirection) -> Option<T>,
    {
        HexNeighbors {
            north: f(HexRowDirection::North),
            north_west: f(HexRowDirection::NorthWest),
            south_west: f(HexRowDirection::SouthWest),
            south: f(HexRowDirection::South),
            south_east: f(HexRowDirection::SouthEast),
            north_east: f(HexRowDirection::NorthEast),
        }
    }
}

impl HexNeighbors<TilePos> {
    /// Returns neighboring tile positions.
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    pub fn get_neighbors(
        tile_pos: &TilePos,
        map_geometry: &Res<MapGeometry>,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(tile_pos);
        let f = |direction| {
            let tile_pos = axial_pos
                .offset_compass_row(direction)
                .as_tile_pos_unchecked();

            map_geometry.check_inclusion(&tile_pos).then_some(tile_pos)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Choose a random neighbor
    pub fn choose_random<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<TilePos> {
        let possible_choices = self.iter().copied().collect::<Vec<TilePos>>();

        possible_choices.choose(rng).cloned()
    }
}

impl<T> HexNeighbors<T> {
    /// Filter using a [`HexNeighbors<bool>`](HexNeighbors). If the filter is `true` for a particular
    /// direction, then return whatever data `self` contains in that direction, otherwise return
    /// `None`.
    ///
    /// `default_none` specifies what boolean value should be associated with the filter if it has
    /// `None` in a given direction. If `default_none` is `true`, then whatever value `self` contains
    /// in that direction will be returned, else `None` will be returned.
    pub fn apply_filter(
        &self,
        hex_filter: &HexNeighbors<MapData<bool>>,
        default_none: bool,
    ) -> HexNeighbors<&T> {
        HexNeighbors::from_directional_closure(|direction| {
            if hex_filter
                .get(direction)
                .map_or(default_none, |filter_data| *filter_data.read())
            {
                self.get(direction)
            } else {
                None
            }
        })
    }
}

impl<T> HexNeighbors<&T>
where
    T: Clone,
{
    /// Clone neighbor data
    pub fn cloned(&self) -> HexNeighbors<T> {
        HexNeighbors {
            north: self.north.cloned(),
            north_west: self.north_west.cloned(),
            south_west: self.south_west.cloned(),
            south: self.south.cloned(),
            south_east: self.south_east.cloned(),
            north_east: self.north_east.cloned(),
        }
    }
}
