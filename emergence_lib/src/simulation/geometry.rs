//! Manages the game world's grid and data tied to that grid

use bevy::{prelude::*, utils::HashMap};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Display, Sub, SubAssign};
use hexx::{shapes::hexagon, Direction, Hex, HexLayout};
use rand::{rngs::ThreadRng, Rng};
use std::f32::consts::PI;

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
        let cubic = self.to_cubic_array();

        let x = cubic[0];
        let y = cubic[1];
        let z = cubic[2];

        write!(f, "({x}, {y}, {z})")
    }
}

impl TilePos {
    /// Generates a new [`TilePos`] from axial coordinates.
    #[cfg(test)]
    pub(crate) fn new(x: i32, y: i32) -> Self {
        TilePos { hex: Hex { x, y } }
    }

    /// Returns the world position (in [`Transform`] units) associated with this tile.
    ///
    /// The `y` value returned corresponds to the top of the tile column at this location.
    #[must_use]
    pub(crate) fn into_world_pos(self, map_geometry: &MapGeometry) -> Vec3 {
        let xz = map_geometry.layout.hex_to_world_pos(self.hex);
        let y = *map_geometry.height_index.get(&self).unwrap();

        Vec3 {
            x: xz.x,
            y,
            z: xz.y,
        }
    }

    /// Returns the nearest tile position to the provided `world_pos`
    ///
    /// `world_pos` generally corresponds to the `translation` of a [`Transform`].
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn from_world_pos(world_pos: Vec3, map_geometry: &MapGeometry) -> Self {
        TilePos {
            hex: map_geometry.layout.world_pos_to_hex(Vec2 {
                x: world_pos.x,
                y: world_pos.z,
            }),
        }
    }

    /// Returns the [`TilePos`] in the provided `direction` from `self`.
    pub(crate) fn neighbor(&self, direction: Direction) -> Self {
        TilePos {
            hex: self.hex.neighbor(direction),
        }
    }

    /// All adjacent tiles that are on the map.
    pub(crate) fn all_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        // PERF: this can be done without any allocations
        let all_hexes = self.hex.all_neighbors();
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
        let neighbors = self.all_neighbors(map_geometry);
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
    /// Which [`Id<Structure>`](crate::asset_management::manifest::Id) entity is stored at each tile position
    pub(crate) structure_index: HashMap<TilePos, Entity>,
    /// Which [`Ghost`](crate::structures::ghost::Ghost) entity is stored at each tile position
    pub(crate) ghost_index: HashMap<TilePos, Entity>,
    /// Which [`Preview`](crate::structures::ghost::Preview) entity is stored at each tile position
    pub(crate) preview_index: HashMap<TilePos, Entity>,
    /// The height of the terrain at each tile position
    pub(crate) height_index: HashMap<TilePos, f32>,
}

impl MapGeometry {
    /// Is the provided `tile_pos` in the map?
    pub(crate) fn is_valid(&self, tile_pos: TilePos) -> bool {
        let distance = Hex::ZERO.distance_to(tile_pos.hex);
        distance <= self.radius as i32
    }

    /// Is the provided `tile_pos` passable?
    ///
    /// Tiles that are not part of the map will return `false`
    pub(crate) fn is_passable(&self, tile_pos: TilePos) -> bool {
        self.is_valid(tile_pos) && !self.structure_index.contains_key(&tile_pos)
    }

    /// Returns the average height of tiles around `tile_pos` within `radius`
    pub(crate) fn average_height(&self, tile_pos: TilePos, radius: u32) -> f32 {
        let hex_iter = hexagon(tile_pos.hex, radius);
        let heights = hex_iter.map(|hex| *self.height_index.get(&TilePos { hex }).unwrap_or(&0.));
        let n = Hex::range_count(radius);
        heights.sum::<f32>() / n as f32
    }

    /// Gets the ghost or structure [`Entity`] at the provided `tile_pos`, if any.
    ///
    /// Ghosts will take priority over structures.
    pub(crate) fn get_ghost_or_structure(&self, tile_pos: TilePos) -> Option<Entity> {
        if let Some(&ghost_entity) = self.ghost_index.get(&tile_pos) {
            Some(ghost_entity)
        } else if let Some(&structure_entity) = self.structure_index.get(&tile_pos) {
            Some(structure_entity)
        } else {
            None
        }
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
            preview_index: HashMap::default(),
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
            direction: Direction::TopRight,
        }
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self.direction {
            Direction::TopRight => "Top-right",
            Direction::Top => "Top",
            Direction::TopLeft => "Top-left",
            Direction::BottomLeft => "Bottom-left",
            Direction::Bottom => "Bottom",
            Direction::BottomRight => "Bottom-right",
        };

        write!(f, "{str}")
    }
}

/// The direction of a [`Facing`] rotation
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub(crate) enum RotationDirection {
    /// Counterclockwise
    Left,
    /// Clockwise
    Right,
}

impl RotationDirection {
    /// Picks a direction to rotate in at random
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen::<bool>() {
            true => RotationDirection::Left,
            false => RotationDirection::Right,
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
        // We want to be aligned with the faces of the hexes, not their points
        let angle = facing.direction.angle(&map_geometry.layout.orientation) + PI / 6.;
        let target = Quat::from_axis_angle(Vec3::Y, angle);
        transform.rotation = target;
    }
}
