//! Manages the game world's grid and data tied to that grid

use bevy::{prelude::*, utils::HashMap};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Display, Sub, SubAssign};
use hexx::{shapes::hexagon, Direction, Hex, HexLayout};
use rand::{rngs::ThreadRng, Rng};
use std::f32::consts::PI;

use crate::filtered_array_iter::FilteredArrayIter;

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
    /// The position of the central tile
    pub const ORIGIN: TilePos = TilePos {
        hex: Hex { x: 0, y: 0 },
    };

    /// Generates a new [`TilePos`] from axial coordinates.
    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        TilePos { hex: Hex { x, y } }
    }

    /// Generates a random [`TilePos`], sampled uniformly from the valid positions in `map_geometry`
    #[inline]
    pub fn random(map_geometry: &MapGeometry, rng: &mut ThreadRng) -> TilePos {
        let range = -(map_geometry.radius as i32)..(map_geometry.radius as i32);

        // Just use rejection sampling: easy to get right
        let mut chosen_tile: Option<TilePos> = None;
        while chosen_tile.is_none() {
            let x = rng.gen_range(range.clone());
            let y = rng.gen_range(range.clone());

            let proposed_tile = TilePos::new(x, y);

            if map_geometry.is_valid(proposed_tile) {
                chosen_tile = Some(proposed_tile)
            }
        }

        chosen_tile.unwrap()
    }

    /// Returns the world position (in [`Transform`] units) associated with this tile.
    ///
    /// The `y` value returned corresponds to the top of the tile column at this location.
    #[must_use]
    pub(crate) fn into_world_pos(self, map_geometry: &MapGeometry) -> Vec3 {
        let xz = map_geometry.layout.hex_to_world_pos(self.hex);
        let y = map_geometry.get_height(self).unwrap().into_world_pos();

        Vec3 {
            x: xz.x,
            y,
            z: xz.y,
        }
    }

    /// Returns the world position (in [`Transform`] units) associated with the top of this tile.
    ///
    /// The `y` value returned corresponds to the top of the tile topper at this location.
    #[must_use]
    pub(crate) fn top_of_tile(self, map_geometry: &MapGeometry) -> Vec3 {
        self.into_world_pos(map_geometry)
            + Vec3 {
                x: 0.,
                y: Height::TOPPER_THICKNESS,
                z: 0.,
            }
    }

    /// Returns the nearest tile position to the provided `world_pos`
    ///
    /// `world_pos` generally corresponds to the `translation` of a [`Transform`].
    #[must_use]
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
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&pos| map_geometry.is_valid(pos));
        iter
    }

    /// All adjacent tiles that are on the map and free of structures.
    pub(crate) fn empty_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&pos| {
            map_geometry.is_valid(pos) && !map_geometry.structure_index.contains_key(&pos)
        });
        iter
    }
}

/// The discretized height of this tile
///
/// The minimum height is 0.
#[derive(
    Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, Display,
)]
pub(crate) struct Height(pub u8);

impl Height {
    /// The minimum allowed height
    pub(crate) const MIN: Height = Height(0);

    /// The maximum allowable height
    pub(crate) const MAX: Height = Height(u8::MAX);

    /// The height of each step up, in world coordinates.
    ///
    /// This should match the thickness of all terrain topper models.
    /// Note that the diameter of a tile is 1.0 transform units.
    pub(crate) const TOPPER_THICKNESS: f32 = 0.224;

    /// Computes the `y` coordinate of a `Transform` that corresponds to this height.
    pub(crate) fn into_world_pos(self) -> f32 {
        self.0 as f32 * Self::TOPPER_THICKNESS
    }

    /// Constructs a new height from the `y` coordinate of a `Transform`.
    ///
    /// Any values outside of the allowable range will be clamped to [`Height::MIN`] and [`Height::MAX`] appropriately.
    pub(crate) fn from_world_pos(world_y: f32) -> Self {
        let f32_height = (world_y / Self::TOPPER_THICKNESS).round();
        if f32_height < 0. {
            Height::MIN
        } else if f32_height > u8::MAX as f32 {
            Height::MAX
        } else if f32_height.is_nan() {
            error!("NaN height conversion detected. Are your transforms broken?");
            Height::MAX
        } else {
            Height(f32_height as u8)
        }
    }

    /// Computes the correct [`Transform`] of the column underneath a tile of this height at this position
    pub(crate) fn column_transform(&self) -> Transform {
        let y_scale = self.into_world_pos();
        let scale = Vec3 {
            x: 1.,
            y: y_scale,
            z: 1.,
        };

        // This is x and z aligned with the parent
        let translation = Vec3 {
            x: 0.,
            // We want this to start below the parent
            y: -y_scale,
            z: 0.,
        };

        Transform {
            translation,
            rotation: Default::default(),
            scale,
        }
    }
}

/// The overall size and arrangement of the map.
#[derive(Debug, Resource)]
pub struct MapGeometry {
    /// The size and orientation of the map.
    pub(crate) layout: HexLayout,
    /// The number of tiles from the center to the edge of the map.
    ///
    /// Note that the central tile is not counted.
    pub(crate) radius: u32,
    /// Which [`Terrain`](crate::asset_management::manifest::Terrain) entity is stored at each tile position
    pub(crate) terrain_index: HashMap<TilePos, Entity>,
    /// Which [`Id<Structure>`](crate::asset_management::manifest::Id) entity is stored at each tile position
    pub(crate) structure_index: HashMap<TilePos, Entity>,
    /// Which [`Ghost`](crate::structures::construction::Ghost) entity is stored at each tile position
    pub(crate) ghost_index: HashMap<TilePos, Entity>,
    /// Which [`Preview`](crate::structures::construction::Preview) entity is stored at each tile position
    pub(crate) preview_index: HashMap<TilePos, Entity>,
    /// The height of the terrain at each tile position
    height_index: HashMap<TilePos, Height>,
}

/// A [`MapGeometry`] index was missing an entry.
#[derive(Debug)]
pub struct IndexError {
    /// The tile position that was missing.
    pub tile_pos: TilePos,
}

impl MapGeometry {
    /// Creates a new [`MapGeometry`] of the provided raidus.
    ///
    /// All indexes will be empty.
    pub fn new(radius: u32) -> Self {
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

    /// Updates the height of the tile at `tile_pos`
    pub(crate) fn update_height(&mut self, tile_pos: TilePos, height: Height) {
        self.height_index.insert(tile_pos, height);
    }

    /// Returns the height of the tile at `tile_pos`, if available.
    ///
    /// This should always be [`Ok`] for all valid tiles.
    pub(crate) fn get_height(&self, tile_pos: TilePos) -> Result<Height, IndexError> {
        match self.height_index.get(&tile_pos) {
            Some(height) => Ok(*height),
            None => Err(IndexError { tile_pos }),
        }
    }

    /// Returns the average height (in world units) of tiles around `tile_pos` within `radius`
    pub(crate) fn average_height(&self, tile_pos: TilePos, radius: u32) -> f32 {
        let hex_iter = hexagon(tile_pos.hex, radius);
        let heights = hex_iter
            .map(|hex| TilePos { hex })
            .filter(|tile_pos| self.is_valid(*tile_pos))
            .map(|tile_pos| {
                let height = self.get_height(tile_pos).unwrap();
                height.into_world_pos()
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_is_invertable() {
        for i in u8::MIN..=u8::MAX {
            let height = Height(i);
            let z = height.into_world_pos();
            let remapped_height = Height::from_world_pos(z);

            assert_eq!(height, remapped_height);
        }
    }

    #[test]
    fn height_clamps() {
        assert_eq!(Height::MIN, Height::from_world_pos(0.));
        assert_eq!(Height::MIN, Height::from_world_pos(-1.));
        assert_eq!(Height::MAX, Height::from_world_pos(9000.));
        assert_eq!(Height::MAX, Height::from_world_pos(f32::MAX));
    }

    #[test]
    fn world_to_tile_pos_conversions_are_invertable() {
        let mut map_geometry = MapGeometry::new(10);

        for x in -10..=10 {
            for y in -10..=10 {
                let tile_pos = TilePos::new(x, y);
                // Height chosen arbitrarily to reduce odds of this accidentally working
                map_geometry.update_height(tile_pos, Height(17));
                let world_pos = tile_pos.into_world_pos(&map_geometry);
                let remapped_tile_pos = TilePos::from_world_pos(world_pos, &map_geometry);

                assert_eq!(tile_pos, remapped_tile_pos);
            }
        }
    }
}
