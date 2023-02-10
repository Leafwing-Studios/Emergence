//! Manages the game world's grid and data tied to that grid

use bevy::{prelude::*, utils::HashMap};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use hexx::{Direction, Hex, HexLayout};

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
pub struct TilePos {
    /// The underlying hex coordinate
    pub hex: Hex,
}

impl TilePos {
    /// Generates a new [`TilePos`] from axial coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        TilePos { hex: Hex { x, y } }
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

impl Default for MapGeometry {
    fn default() -> Self {
        MapGeometry {
            layout: HexLayout::default(),
            radius: 50,
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
pub struct Facing {
    /// The desired direction.
    ///
    /// Defaults to [`Direction::Top`].
    pub direction: Direction,
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
    mut query: Query<(&mut Transform, &Facing), (Changed<Facing>, Without<Camera3d>)>,
    map_geometry: Res<MapGeometry>,
) {
    for (mut transform, &facing) in query.iter_mut() {
        // Rotate the object in the correct direction
        let angle = facing.direction.angle(&map_geometry.layout.orientation);
        let target = Quat::from_axis_angle(Vec3::Y, angle);
        transform.rotation = target;
    }
}
