//! Manages the game world's grid and data tied to that grid

use bevy::{prelude::*, utils::HashMap};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use hexx::{Direction, Hex, HexLayout};
use rand::{prelude::IteratorRandom, rngs::ThreadRng};

/// A hex-based coordinate, that represents exactly one tile.
#[derive(
    Component,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deref,
    DerefMut,
    Default,
    Add,
    Sub,
    AddAssign,
    SubAssign,
)]
pub(crate) struct TilePos {
    /// The underlying hex coordinate
    pub(crate) hex: Hex,
}

impl Display for TilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.hex.x;
        let y = self.hex.y;
        // In cubic hex coordinates, x+y+z = 0
        let z = -x - y;

        write!(f, "({x}, {y}, {z})")
    }
}

impl TilePos {
    /// Generates a new [`TilePos`] from axial coordinates.
    #[cfg(test)]
    pub(crate) fn new(x: i32, y: i32) -> Self {
        TilePos { hex: Hex { x, y } }
    }

    /// Choose a random adjacent tile without structures.
    ///
    /// It must be free of structures and on the map.
    /// Returns [`None`] if no viable options exist.
    pub(crate) fn choose_random_empty_neighbor(
        &self,
        rng: &mut ThreadRng,
        map_geometry: &MapGeometry,
    ) -> Option<TilePos> {
        let empty_neighbors = self.empty_neighbors(map_geometry);

        empty_neighbors.into_iter().choose(rng)
    }

    /// All adjacent tiles that are on the map.
    pub(crate) fn neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        // PERF: this can be done without any allocations
        let all_hexes = self.all_neighbors();
        let mut neighbors = Vec::new();

        for &hex in all_hexes.iter() {
            let tile_pos = TilePos { hex };
            if map_geometry.is_valid(tile_pos) {
                neighbors.push(tile_pos);
            }
        }

        neighbors
    }

    /// All adjacent tiles that are on the map and free of structures.
    pub(crate) fn empty_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.neighbors(map_geometry);
        // PERF: this can be done without allocations
        let empty_neighbors: Vec<TilePos> = neighbors
            .into_iter()
            .filter(|tile_pos| !map_geometry.structure_index.contains_key(tile_pos))
            .collect();

        empty_neighbors
    }
}

/// The overall size and arrangement of the map.
#[derive(Debug, Resource)]
pub(crate) struct MapGeometry {
    /// The size and orientation of the map.
    pub(crate) layout: HexLayout,
    /// The number of tiles from the center to the edge of the map.
    ///
    /// Note that the central tile is not counted.
    pub(crate) radius: u32,
    /// Which [`Terrain`](crate::terrain::Terrain) entity is stored at each tile position
    pub(crate) terrain_index: HashMap<TilePos, Entity>,
    /// Which [`StructureId`](crate::structures::StructureId) entity is stored at each tile position
    pub(crate) structure_index: HashMap<TilePos, Entity>,
    /// Which [`Ghost`](crate::structures::ghost::Ghost) entity is stored at each tile position
    pub(crate) ghost_index: HashMap<TilePos, Entity>,
    /// The height of the terrain at each tile position
    pub(crate) height_index: HashMap<TilePos, f32>,
}

impl MapGeometry {
    /// Is the provided `tile_pos` in the map?
    pub(crate) fn is_valid(&self, tile_pos: TilePos) -> bool {
        let distance = Hex::ZERO.distance_to(tile_pos.hex);
        distance <= self.radius as i32
    }
}

impl MapGeometry {
    /// Initializes the geometry for a new map.
    pub(super) fn new(radius: u32) -> Self {
        MapGeometry {
            layout: HexLayout::default(),
            radius,
            terrain_index: HashMap::default(),
            structure_index: HashMap::default(),
            ghost_index: HashMap::default(),
            height_index: HashMap::default(),
        }
    }
}

/// The hex direction that this entity is facing.
///
/// Stored as a component on each entity with a grid-aligned rotation.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut)]
pub(crate) struct Facing {
    /// The desired direction.
    ///
    /// Defaults to [`Direction::Top`].
    pub direction: Direction,
}

impl Facing {
    /// Rotates this facing one 60 degree step clockwise.
    pub(crate) fn rotate_left(&mut self) {
        self.direction = self.direction.left();
    }

    /// Rotates this facing one 60 degree step counterclockwise.
    pub(crate) fn rotate_right(&mut self) {
        self.direction = self.direction.right();
    }
}

impl Default for Facing {
    fn default() -> Self {
        Facing {
            direction: Direction::Top,
        }
    }
}

/// Rotates objects so they are facing the correct direction.
pub(super) fn sync_rotation_to_facing(
    // Camera requires different logic, it rotates "around" a central point
    // PERF: re-enable change detection. For some reason this wasn't working on structures,
    // but was on ghosts.
    mut query: Query<(&mut Transform, &Facing), Without<Camera3d>>,
    map_geometry: Res<MapGeometry>,
) {
    for (mut transform, &facing) in query.iter_mut() {
        // Rotate the object in the correct direction
        let angle = facing.direction.angle(&map_geometry.layout.orientation);
        let target = Quat::from_axis_angle(Vec3::Y, angle);
        transform.rotation = target;
    }
}
