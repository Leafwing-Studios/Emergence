//! Code for managing data that is deeply tied to the map

use crate::simulation::map::{MapGeometry, MapPositions};
use bevy::prelude::{Res, Resource};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::{HexRowDirection, HEX_DIRECTIONS};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Spatial data for use with the [`MapResource`] struct.
#[derive(Default, Clone)]
pub struct MapData<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> MapData<T>
where
    T: Default,
{
    /// Create from data
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }

    /// Immutably the inner data
    pub fn borrow(&self) -> &T {
        self.inner.borrow()
    }

    /// Mutably borrow the inner data
    pub fn borrow_mut(&mut self) -> &mut T {
        self.inner.get_mut().unwrap()
    }
}

/// A helper for managing game resources that are naturally tied to a fixed specific position on
/// the map
///
/// It can give you [`MapData<T>`](MapData) at a given tile position, or it can give you
/// [`HexNeighbors<MapData<T>>`](HexNeighbors) for the given position.
///
/// Internally, [`MapData`] is stored in a [`HashMap`] for each position, in the `storage` field,
/// and this same data is then referenced to by the `neighbors` field.
#[derive(Resource)]
pub struct MapResource<T> {
    storage: HashMap<TilePos, MapData<T>>,
    neighbors: HashMap<TilePos, HexNeighbors<MapData<T>>>,
}

impl<T> MapResource<T>
where
    T: Default,
{
    /// Create new from an underlying `MapPostions` template
    ///
    /// This allocates capacity and initializes neighbors based on the template provided.
    ///
    /// This requires that that there is a `Default` impl for the underlying data type
    pub fn default_from_template(template: &MapPositions) -> MapResource<T> {
        let n_positions = template.n_positions();

        let mut storage = HashMap::with_capacity(n_positions);
        storage.extend(
            template
                .iter_positions()
                .map(|position| (*position, MapData::new(T::default()))),
        );

        let mut neighbors = HashMap::with_capacity(n_positions);
        neighbors.extend(template.iter_neighbors().filter_map(|position| {
            let tile_neighbors = template.get_neighbors(position)?;
            let neighbors = tile_neighbors.and_then(|position| storage.get(position).cloned());
            Some((*position, neighbors))
        }));

        MapResource { storage, neighbors }
    }
}

impl<T> MapResource<T> {
    /// Create new from an underlying `MapPostions` template.
    ///
    /// This allocates capacity and initializes neighbors based on the template provided.
    ///
    /// If your underlying data implements [`Default`], you could use
    /// [`default_from_template`](MapData::default_from_template) to also initialize data.
    pub fn new(
        template: &MapPositions,
        data: impl Iterator<Item = (TilePos, T)>,
    ) -> MapResource<T> {
        let n_positions = template.n_positions();

        let mut storage = HashMap::with_capacity(n_positions);
        storage.extend(data);

        let mut neighbors = HashMap::with_capacity(n_positions);
        neighbors.extend(template.iter_neighbors().filter_map(|position| {
            let tile_neighbors = template.get_neighbors(position)?;
            let neighbors = tile_neighbors.and_then(|position| storage.get(position).cloned());
            Some((*position, neighbors))
        }));

        MapResource { storage, neighbors }
    }

    /// Replace data at the specified position
    pub fn replace(&mut self, position: &TilePos, replace_with: T) {
        self.storage.get(position).unwrap().borrow_mut() = replace_with;
    }

    /// Get data stored at given position
    pub fn get(&self, pos: &TilePos) -> Option<MapData<T>> {
        self.storage.get(pos).cloned()
    }

    /// Get neighbor data for given position
    pub fn get_neighbors(&self, pos: &TilePos) -> Option<&HexNeighbors<MapData<T>>> {
        self.neighbors.get(pos)
    }

    /// Iterate over the positions managed by this resource
    pub fn positions(&self) -> impl Iterator<Item = &TilePos> {
        self.storage.keys()
    }

    /// Iterate over the data at all positions
    pub fn values(&self) -> impl Iterator<Item = MapData<T>> {
        self.storage.values()
    }
}

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
            HexRowDirection::North => self.north.as_ref_mut(),
            HexRowDirection::NorthWest => self.north_west.as_ref_mut(),
            HexRowDirection::SouthWest => self.south_west.as_ref_mut(),
            HexRowDirection::South => self.south.as_ref_mut(),
            HexRowDirection::SouthEast => self.south_east.as_ref_mut(),
            HexRowDirection::NorthEast => self.north_east.as_ref_mut(),
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
