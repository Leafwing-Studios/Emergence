use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::{
    HexDirection, HexRowDirection, HEX_DIRECTIONS,
};
use bevy_ecs_tilemap::prelude::TilemapSize;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;

/// Generates a random hexagonal direction using the `rng` and `distribution` provided.
#[allow(unused)]
fn random_direction<R: Rng + ?Sized, D: Distribution<usize>>(
    mut rng: &mut R,
    distribution: D,
) -> HexDirection {
    let choice = distribution.sample(&mut rng);
    HexDirection::from(choice)
}

/// Stores some copy-able data `T` associated with each neighboring hex cell, if present.
#[derive(Debug, Default)]
pub struct HexNeighbors<T> {
    north_west: Option<T>,
    west: Option<T>,
    south_west: Option<T>,
    south_east: Option<T>,
    east: Option<T>,
    north_east: Option<T>,
}

impl<T> HexNeighbors<T> {
    pub fn get(&self, direction: HexRowDirection) -> Option<&T> {
        use HexRowDirection::*;
        match direction {
            East => self.east.as_ref(),
            NorthEast => self.north_east.as_ref(),
            NorthWest => self.north_west.as_ref(),
            West => self.west.as_ref(),
            SouthWest => self.south_west.as_ref(),
            SouthEast => self.south_east.as_ref(),
        }
    }

    pub fn set(&mut self, direction: HexRowDirection, data: T) {
        use HexRowDirection::*;
        match direction {
            East => {
                self.east.replace(data);
            }
            NorthEast => {
                self.north_east.replace(data);
            }
            NorthWest => {
                self.north_west.replace(data);
            }
            West => {
                self.west.replace(data);
            }
            SouthWest => {
                self.south_west.replace(data);
            }
            SouthEast => {
                self.south_east.replace(data);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        HEX_DIRECTIONS
            .into_iter()
            .filter_map(|direction| self.get(direction.into()))
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// element, where `f` takes `T` by value.
    pub fn and_then<U: Copy, F>(&self, f: F) -> HexNeighbors<U>
    where
        F: Fn(T) -> Option<U>,
    {
        HexNeighbors {
            north_west: self.north_west.and_then(&f),
            west: self.west.and_then(&f),
            south_west: self.south_west.and_then(&f),
            south_east: self.south_east.and_then(&f),
            east: self.east.and_then(&f),
            north_east: self.north_east.and_then(&f),
        }
    }

    /// Generates `HexNeighbors<T>` from a closure that takes a hex direction, and outputs
    /// `Option<T>`.
    pub fn from_directional_closure<F>(f: F) -> HexNeighbors<T>
    where
        F: Fn(HexRowDirection) -> Option<T>,
    {
        use HexRowDirection::*;
        HexNeighbors {
            north_west: f(NorthWest),
            west: f(West),
            south_west: f(SouthWest),
            south_east: f(SouthEast),
            east: f(East),
            north_east: f(NorthEast),
        }
    }
}

impl HexNeighbors<TilePos> {
    /// Returns the set of neighboring cells
    pub fn get_neighbors(tile_pos: &TilePos, map_size: &TilemapSize) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(tile_pos);
        let f = |direction| {
            axial_pos
                .offset_compass_row(direction)
                .as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    pub fn entities(&self, tile_storage: &TileStorage) -> HexNeighbors<Entity> {
        let f = |tile_pos| tile_storage.get(&tile_pos);
        self.and_then(f)
    }

    /// Choose a random neighbor
    pub fn choose_random<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<TilePos> {
        let possible_choices = self.iter().copied().collect::<Vec<TilePos>>();

        possible_choices.choose(rng).cloned()
    }
}
