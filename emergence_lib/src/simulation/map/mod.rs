//! Manages the game world's grid and data tied to that grid

use crate::simulation::generation::GenerationConfig;
use bevy::ecs::system::Resource;
use bevy::log::info;
use bevy::prelude::{Commands, Component, Deref, DerefMut, Res};
use hexx::Hex;

/// A hex-based coordinate, that represents exactly one tile.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct TilePos {
    /// The underlying hex coordinate
    pub hex: Hex,
}

/// Resource that stores information regarding the size of the game map.
#[derive(Resource, Debug)]
pub struct MapGeometry {
    /// The radius, in tiles, of the map
    radius: u32,
    /// The location of the central tile
    center: TilePos,
}

impl Default for MapGeometry {
    fn default() -> Self {
        MapGeometry::new(GenerationConfig::MAP_RADIUS)
    }
}

impl MapGeometry {
    /// Constructs a new [`MapGeometry`] for a `radius`.
    pub const fn new(radius: u32) -> Self {
        MapGeometry {
            radius,
            center: TilePos { hex: Hex::ZERO },
        }
    }

    /// Computes the number of positions that exist in this map
    pub const fn n_positions(&self) -> usize {
        1 + 6 * ((self.radius * (self.radius + 1)) / 2) as usize
    }

    /// Computes the [`TilePos`] of the tile at the center of this map.
    #[inline]
    pub const fn center(&self) -> TilePos {
        self.center
    }

    /// Gets the radius of the map
    #[inline]
    pub const fn radius(&self) -> u32 {
        self.radius
    }
}

/// Initialize the [`MapGeometry`] resource according to [`GenerationConfig`].
pub fn configure_map_geometry(mut commands: Commands, config: Res<GenerationConfig>) {
    info!("Configuring map geometry...");
    let map_geometry: MapGeometry = MapGeometry::new(config.map_radius);

    commands.insert_resource(map_geometry);
}

/// Resource caching tile positions for a fixed map size
#[derive(Resource, Default)]
pub struct MapPositions {
    /// Number of positions that are expected given the size of teh hexagonal map
    n_positions: usize,
    /// Vector of positions that exist in the map
    positions: Vec<TilePos>,
}

impl MapPositions {
    /// Creates map positions for a hexagonal map specified by the given [`MapGeometry`]
    pub fn new(map_geometry: &MapGeometry) -> MapPositions {
        let n_positions = map_geometry.n_positions();

        let mut map_positions = MapPositions {
            n_positions,
            positions: Vec::with_capacity(n_positions),
        };

        let center = map_geometry.center();
        let radius = map_geometry.radius();
        for position in generate_hexagon(AxialPos::from(&center), radius)
            .into_iter()
            .map(|axial_pos| axial_pos.as_tile_pos_unchecked())
        {
            map_positions.positions.push(position);
        }

        map_positions
    }

    /// Get an iterator over tile positions
    pub fn iter_positions(&self) -> impl Iterator<Item = &TilePos> + '_ {
        self.positions.iter()
    }

    /// Get the number of positions that are managed by this structure
    pub const fn n_positions(&self) -> usize {
        self.n_positions
    }
}

/// Create the [`MapPositions`] resource
pub fn create_map_positions(mut commands: Commands, map_geometry: Res<MapGeometry>) {
    info!("Creating map positions cache...");
    let map_positions = MapPositions::new(&map_geometry);

    commands.insert_resource(map_positions);
}
