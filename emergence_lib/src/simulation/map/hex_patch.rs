//! Code for managing data tied to (spatial) neighbors of a given position

use crate as emergence_lib;
use crate::enum_iter::IterableEnum;
use crate::simulation::map::index::MapData;
use crate::simulation::map::MapGeometry;
use emergence_macros::IterableEnum;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Debug;

use super::TilePos;

/// Enumerates the positions in a 7-tile hex patch (central tile + 6 neighbors)
#[derive(Debug, Clone, Copy, Hash, IterableEnum)]
pub enum HexPatchLocation {
    /// The central position
    Center,
    /// The northern (neighboring) position
    North,
    /// The north-western (neighboring) position
    NorthWest,
    /// The south-western (neighboring) position
    SouthWest,
    /// The southern (neighboring) position
    South,
    /// The south-eastern (neighboring) position
    SouthEast,
    /// The north-eastern (neighboring) position
    NorthEast,
}

impl HexPatchLocation {
    /// Convert a non-central hex patch location into a [`HexRowDirection`]
    pub fn as_hex_row_direction(&self) -> Option<HexRowDirection> {
        match self {
            HexPatchLocation::Center => None,
            HexPatchLocation::North => Some(HexRowDirection::North),
            HexPatchLocation::NorthWest => Some(HexRowDirection::NorthWest),
            HexPatchLocation::SouthWest => Some(HexRowDirection::SouthWest),
            HexPatchLocation::South => Some(HexRowDirection::South),
            HexPatchLocation::SouthEast => Some(HexRowDirection::SouthEast),
            HexPatchLocation::NorthEast => Some(HexRowDirection::NorthEast),
        }
    }
}

/// Stores some data `T` associated with each neighboring hex cell, if present.
#[derive(Debug)]
pub struct HexPatch<T> {
    /// The central tile
    center: Option<T>,
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

impl<T> Default for HexPatch<T> {
    fn default() -> Self {
        HexPatch {
            center: None,
            north: None,
            north_west: None,
            south_west: None,
            south: None,
            south_east: None,
            north_east: None,
        }
    }
}

impl<T> HexPatch<T> {
    /// Get the neighbor in the specified direction.
    pub fn get(&self, location: HexPatchLocation) -> Option<&T> {
        match location {
            HexPatchLocation::Center => self.center.as_ref(),
            HexPatchLocation::North => self.north.as_ref(),
            HexPatchLocation::NorthWest => self.north_west.as_ref(),
            HexPatchLocation::SouthWest => self.south_west.as_ref(),
            HexPatchLocation::South => self.south.as_ref(),
            HexPatchLocation::SouthEast => self.south_east.as_ref(),
            HexPatchLocation::NorthEast => self.north_east.as_ref(),
        }
    }

    /// Get mutable access to neighbor in the specified direction.
    pub fn get_mut(&mut self, location: HexPatchLocation) -> &mut Option<T> {
        match location {
            HexPatchLocation::Center => &mut self.center,
            HexPatchLocation::North => &mut self.north,
            HexPatchLocation::NorthWest => &mut self.north_west,
            HexPatchLocation::SouthWest => &mut self.south_west,
            HexPatchLocation::South => &mut self.south,
            HexPatchLocation::SouthEast => &mut self.south_east,
            HexPatchLocation::NorthEast => &mut self.north_east,
        }
    }

    /// Get a mutable reference to the neighbor in the specified direction.
    pub fn get_inner_mut(&mut self, location: HexPatchLocation) -> Option<&mut T> {
        match location {
            HexPatchLocation::Center => self.center.as_mut(),
            HexPatchLocation::North => self.north.as_mut(),
            HexPatchLocation::NorthWest => self.north_west.as_mut(),
            HexPatchLocation::SouthWest => self.south_west.as_mut(),
            HexPatchLocation::South => self.south.as_mut(),
            HexPatchLocation::SouthEast => self.south_east.as_mut(),
            HexPatchLocation::NorthEast => self.north_east.as_mut(),
        }
    }

    /// Iterate through positions with [`Some`] data
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        HexPatchLocation::variants().filter_map(|location| self.get(location))
    }

    /// Count the number of positions (center and neighbors) with [`Some`] data
    pub fn count(&self) -> usize {
        self.iter().count()
    }

    /// Applies the supplied closure `f` with an [`and_then`](Option::and_then) to each
    /// neighbor element, where `f` takes `T` by value.
    pub fn and_then<U, F>(self, f: F) -> HexPatch<U>
    where
        F: Fn(T) -> Option<U>,
    {
        HexPatch {
            center: self.center.and_then(&f),
            north: self.north.and_then(&f),
            north_west: self.north_west.and_then(&f),
            south_west: self.south_west.and_then(&f),
            south: self.south.and_then(&f),
            south_east: self.south_east.and_then(&f),
            north_east: self.north_east.and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with an [`and_then`](Option::and_then) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn and_then_ref<'a, U, F>(&'a self, f: F) -> HexPatch<U>
    where
        F: Fn(&'a T) -> Option<U>,
    {
        HexPatch {
            center: self.center.as_ref().and_then(&f),
            north: self.north.as_ref().and_then(&f),
            north_west: self.north_west.as_ref().and_then(&f),
            south_west: self.south_west.as_ref().and_then(&f),
            south: self.south.as_ref().and_then(&f),
            south_east: self.south_east.as_ref().and_then(&f),
            north_east: self.north_east.as_ref().and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with a [`map`](Option::map) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn map_ref<'a, U, F>(&'a self, f: F) -> HexPatch<U>
    where
        F: Fn(&'a T) -> U,
    {
        HexPatch {
            center: self.center.as_ref().map(&f),
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
    pub fn from_locational_closure<F>(f: F) -> HexPatch<T>
    where
        F: Fn(HexPatchLocation) -> Option<T>,
    {
        let mut result: HexPatch<T> = HexPatch::default();
        HexPatchLocation::variants().for_each(|location| {
            *(result.get_mut(location)) = f(location);
        });
        result
    }
}

impl HexPatch<TilePos> {
    /// Generates a hex patch of positions centered on the specified center position
    ///
    /// A position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    pub fn generate(center_position: &TilePos, map_geometry: &MapGeometry) -> HexPatch<TilePos> {
        let axial_pos = AxialPos::from(center_position);
        let f = |location| {
            let tile_pos = match location {
                HexPatchLocation::Center => *center_position,
                _ => axial_pos
                    .offset_compass_row(location.as_hex_row_direction().unwrap())
                    .as_tile_pos_unchecked(),
            };

            map_geometry.check_inclusion(&tile_pos).then_some(tile_pos)
        };
        HexPatch::from_locational_closure(f)
    }

    /// Choose a random neighbor
    pub fn choose_random<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<TilePos> {
        let possible_choices = self.iter().copied().collect::<Vec<TilePos>>();

        possible_choices.choose(rng).cloned()
    }
}

impl<T> HexPatch<T> {
    /// Filter using a [`HexNeighbors<bool>`](HexPatch). If the filter is `true` for a particular
    /// direction, then return whatever data `self` contains in that direction, otherwise return
    /// `None`.
    ///
    /// `default_none` specifies what boolean value should be associated with the filter if it has
    /// `None` in a given direction. If `default_none` is `true`, then whatever value `self` contains
    /// in that direction will be returned, else `None` will be returned.
    pub fn apply_filter(
        &self,
        filter_patch: &HexPatch<MapData<bool>>,
        default_none: bool,
    ) -> HexPatch<&T> {
        HexPatch::from_locational_closure(|location| {
            if filter_patch
                .get(location)
                .map_or(default_none, |filter_data| *filter_data.read())
            {
                self.get(location)
            } else {
                None
            }
        })
    }
}

impl<T> HexPatch<&T>
where
    T: Clone,
{
    /// Clone neighbor data
    pub fn cloned(&self) -> HexPatch<T> {
        HexPatch {
            center: self.center.cloned(),
            north: self.north.cloned(),
            north_west: self.north_west.cloned(),
            south_west: self.south_west.cloned(),
            south: self.south.cloned(),
            south_east: self.south_east.cloned(),
            north_east: self.north_east.cloned(),
        }
    }
}
