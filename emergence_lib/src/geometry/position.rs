//! Types for positioning and measuring coordinates.

use bevy::{math::Vec3Swizzles, prelude::*};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use hexx::{Direction, Hex};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Formatter,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use crate::filtered_array_iter::FilteredArrayIter;

use super::{Facing, MapGeometry};

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
    Serialize,
    Deserialize,
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
    pub const ZERO: TilePos = TilePos {
        hex: Hex { x: 0, y: 0 },
    };

    /// Generates a new [`TilePos`] from axial coordinates.
    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        TilePos { hex: Hex { x, y } }
    }

    /// Generates a random [`TilePos`], sampled uniformly from the valid positions in `map_geometry`
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
    pub(crate) fn into_world_pos(self, map_geometry: &MapGeometry) -> Vec3 {
        let xz = map_geometry.layout.hex_to_world_pos(self.hex);
        let y = map_geometry
            .get_height(self)
            .unwrap_or_default()
            .into_world_pos();

        Vec3 {
            x: xz.x,
            y,
            z: xz.y,
        }
    }

    /// Returns the world position (in [`Transform`] units) associated with the top of this tile.
    ///
    /// The `y` value returned corresponds to the top of the tile topper at this location.
    #[inline]
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
    #[inline]
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
    #[inline]
    #[must_use]
    pub(crate) fn neighbor(&self, direction: Direction) -> Self {
        TilePos {
            hex: self.hex.neighbor(direction),
        }
    }

    /// All neighbors of `self`.
    ///
    /// # Warning
    ///
    /// This includes neighbors that are not on the map.
    #[inline]
    #[must_use]
    pub(crate) fn all_neighbors(&self) -> impl IntoIterator<Item = TilePos> {
        self.hex.all_neighbors().map(|hex| TilePos { hex })
    }

    /// All adjacent tiles that are clear of structures and ghosts and have a limited terrain height gap.
    ///
    /// This is used to control vegetative reproduction.
    pub(crate) fn empty_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&tile_pos| {
            map_geometry.is_valid(tile_pos)
                && map_geometry.get_ghost_structure(tile_pos).is_none()
                && map_geometry.get_ghost_terrain(tile_pos).is_none()
                && map_geometry.get_structure(tile_pos).is_none()
        });
        iter
    }

    /// Returns the [`TilePos`] rotated to match the `facing` around the origin.
    #[inline]
    #[must_use]
    pub(crate) fn rotated(&self, facing: Facing) -> Self {
        let n_rotations = facing.rotation_count();

        TilePos {
            // This must rotate counter-clockwise,
            // as we are rotating the tile around the origin.
            hex: self.hex.rotate_ccw(n_rotations),
        }
    }

    /// Computes the flat distance between the centers of self and `other` in world coordinates.
    ///
    /// Note that this is not the same as the distance between tiles in tile coordinates!
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn world_space_distance(&self, other: TilePos, map_geometry: &MapGeometry) -> f32 {
        let self_pos = self.into_world_pos(map_geometry).xz();
        let other_pos = other.into_world_pos(map_geometry).xz();

        self_pos.distance(other_pos)
    }

    /// Computes the length of the shortest path between the centers of self and `other` in tile coordinates.
    ///
    /// Note that this is not the same as the distance between tiles in world coordinates!
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn manhattan_tile_distance(&self, other: TilePos) -> f32 {
        (self.hex - other.hex).length() as f32
    }

    /// Computes the Euclidean distance between the centers of self and `other` in tile coordinates.
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn euclidean_tile_distance(&self, other: TilePos) -> f32 {
        let [a_x, a_y, a_z] = self.hex.to_cubic_array();
        let [b_x, b_y, b_z] = other.hex.to_cubic_array();

        let dist_sq = ((a_x - b_x).pow(2) + (a_y - b_y).pow(2) + (a_z - b_z).pow(2)) as f32;
        dist_sq.sqrt()
    }
}

impl Mul<i32> for TilePos {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        TilePos {
            hex: self.hex * rhs,
        }
    }
}

/// The discretized height of this tile
///
/// The minimum height is 0.
#[derive(Component, Clone, Copy, Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Height(pub f32);

impl Display for Height {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Height {
    /// The absolute minimum height.
    pub(crate) const ZERO: Height = Height(0.);

    /// The minimum allowed height
    pub(crate) const MIN: Height = Height(0.);

    /// The maximum allowable height
    pub(crate) const MAX: Height = Height(255.);

    /// The maximum height difference that units can traverse in a single step.
    pub(crate) const MAX_STEP: Height = Height(1.);

    /// The thickness of all terrain topper models in world coordinates.
    /// Note that the diameter of a tile is 1.0 transform units.
    pub(crate) const TOPPER_THICKNESS: f32 = 0.224;

    /// The height of each step up, in world coordinates.
    pub(crate) const STEP_HEIGHT: f32 = 1.0;

    /// The maximum height of water that units can walk through.
    pub(crate) const WADING_DEPTH: Height = Height(1.);

    /// Computes the `y` coordinate of a `Transform` that corresponds to this height.
    #[inline]
    #[must_use]
    pub(crate) fn into_world_pos(self) -> f32 {
        self.0 * Self::STEP_HEIGHT
    }

    /// Constructs a new height from the `y` coordinate of a `Transform`.
    ///
    /// Any values outside of the allowable range will be clamped to [`Height::MIN`] and [`Height::MAX`] appropriately.
    #[inline]
    #[must_use]
    pub(crate) fn from_world_pos(world_y: f32) -> Self {
        let height = (world_y / Self::STEP_HEIGHT).round();
        if height < 0. {
            Height::MIN
        } else if height > u8::MAX as f32 {
            Height::MAX
        } else if height.is_nan() {
            error!("NaN height conversion detected. Are your transforms broken?");
            Height::MAX
        } else {
            Height(height)
        }
    }

    /// Computes the correct [`Transform`] of the column underneath a tile of this height at this position
    #[inline]
    #[must_use]
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

    /// Raises the height by a single terrain step.
    ///
    /// Clamps the height to [`Height::MIN`] if it would go below it.
    /// Clamps the height to [`Height::MAX`] if it would exceed it.
    /// Rounds the height to the nearest integer.
    #[inline]
    pub(crate) fn raise(&mut self) {
        self.0 = (self.0 + 1.).clamp(Height::MIN.0, Height::MAX.0).round();
    }

    /// Lowers the height by a single terrain step.
    ///
    /// Clamps the height to [`Height::MIN`] if it would go below it.
    /// Clamps the height to [`Height::MAX`] if it would exceed it.
    /// Rounds the height to the nearest integer.
    #[inline]
    pub(crate) fn lower(&mut self) {
        self.0 = (self.0 - 1.).clamp(Height::MIN.0, Height::MAX.0).round();
    }

    /// Returns the lower of the two heights.
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn min(self, other: Self) -> Self {
        Height(self.0.min(other.0))
    }

    /// Returns the higher of the two heights.
    #[inline]
    #[must_use]
    pub(crate) fn max(self, other: Self) -> Self {
        Height(self.0.max(other.0))
    }

    /// Returns the absolute difference between the two heights.
    #[inline]
    #[must_use]
    pub(crate) fn abs_diff(self, other: Self) -> Self {
        Height((self.0 - other.0).abs())
    }
}

impl Add for Height {
    type Output = Height;

    fn add(self, rhs: Self) -> Self::Output {
        Height(self.0 + rhs.0)
    }
}

impl Sub for Height {
    type Output = Height;

    fn sub(self, rhs: Self) -> Self::Output {
        Height((self.0 - rhs.0).max(0.))
    }
}

impl AddAssign for Height {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Height {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f32> for Height {
    type Output = Height;

    fn mul(self, rhs: f32) -> Self::Output {
        Height(self.0 * rhs)
    }
}

impl Mul<Height> for f32 {
    type Output = Height;

    fn mul(self, rhs: Height) -> Self::Output {
        Height(self * rhs.0)
    }
}

impl Div<f32> for Height {
    type Output = Height;

    fn div(self, rhs: f32) -> Self::Output {
        Height(self.0 / rhs)
    }
}

/// A voxel position in the game world.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Default)]
pub struct VoxelPos {
    /// Corresponds to the x coordinate of the [`TilePos`]
    x: i32,
    /// Corresponds to the y coordinate of the [`TilePos`]
    y: i32,
    /// The discretized [`Height`] of the voxel.
    height: i32,
}

impl VoxelPos {
    /// Create a new [`VoxelPos`] from a [`TilePos`] and a [`Height`].
    pub fn new(tile_pos: TilePos, height: Height) -> Self {
        Self {
            x: tile_pos.x,
            y: tile_pos.y,
            height: height.0.round() as i32,
        }
    }

    /// Get the [`TilePos`] corresponding to this [`VoxelPos`].
    pub fn tile_pos(&self) -> TilePos {
        TilePos::new(self.x, self.y)
    }

    /// Get the [`Height`] of this [`VoxelPos`].
    pub fn height(&self) -> Height {
        Height(self.height as f32)
    }

    /// Gets the voxel position of the voxel above this one.
    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            height: self.height + 1,
        }
    }

    /// Gets the voxel position of the voxel below this one.
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            height: self.height - 1,
        }
    }
}

/// A volume of space, in tile units.
///
/// A value of 1.0 represents the volume of a single tile.
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    Reflect,
    Add,
    Sub,
    AddAssign,
    SubAssign,
)]
pub struct Volume(pub f32);

impl Volume {
    /// The empty volume.
    pub const ZERO: Volume = Volume(0.);

    /// The volume of a single tile.
    pub const ONE: Volume = Volume(1.);

    /// Computes the volume of the provided area and height.
    #[inline]
    #[must_use]
    pub fn from_area_and_height(n_tiles: usize, height: Height) -> Self {
        Volume(n_tiles as f32 * height.0)
    }

    /// Computes the volume of a single tile with the provided height.
    #[inline]
    #[must_use]
    pub fn from_height(height: Height) -> Self {
        Volume(height.0)
    }

    /// Computes the height of a single tile with the provided volume.
    #[inline]
    #[must_use]
    pub fn into_height(self) -> Height {
        Height(self.0)
    }

    /// Returns the lower of the two volumes.
    #[inline]
    #[must_use]
    pub(crate) fn min(self, other: Self) -> Self {
        Volume(self.0.min(other.0))
    }

    /// Returns the higher of the two volumes.
    #[inline]
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Volume(self.0.max(other.0))
    }

    /// Computes the absolute difference between the two volumes.
    #[inline]
    #[must_use]
    pub fn abs_diff(self, other: Self) -> Self {
        Volume((self.0 - other.0).abs())
    }
}

impl Display for Volume {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Mul<f32> for Volume {
    type Output = Volume;

    fn mul(self, rhs: f32) -> Self::Output {
        Volume(self.0 * rhs)
    }
}

impl Mul<Volume> for f32 {
    type Output = Volume;

    fn mul(self, rhs: Volume) -> Self::Output {
        Volume(self * rhs.0)
    }
}

impl Div<f32> for Volume {
    type Output = Volume;

    fn div(self, rhs: f32) -> Self::Output {
        Volume(self.0 / rhs)
    }
}

impl Div<Volume> for Volume {
    type Output = f32;

    fn div(self, rhs: Volume) -> Self::Output {
        self.0 / rhs.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_is_invertable() {
        for i in u8::MIN..=u8::MAX {
            let height = Height(i as f32);
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
        let mut map_geometry = MapGeometry::new(20);

        for x in -10..=10 {
            for y in -10..=10 {
                let tile_pos = TilePos::new(x, y);
                // Height chosen arbitrarily to reduce odds of this accidentally working
                map_geometry.update_height(tile_pos, Height(17.));
                let world_pos = tile_pos.into_world_pos(&map_geometry);
                let remapped_tile_pos = TilePos::from_world_pos(world_pos, &map_geometry);

                assert_eq!(tile_pos, remapped_tile_pos);
            }
        }
    }
}
