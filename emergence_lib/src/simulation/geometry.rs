//! Manages the game world's grid and data tied to that grid

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::{HashMap, HashSet},
};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Display, Sub, SubAssign};
use hexx::{shapes::hexagon, ColumnMeshBuilder, Direction, Hex, HexLayout, HexOrientation};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::{
    f32::consts::{PI, TAU},
    fmt::Formatter,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use crate::{
    asset_management::manifest::Id,
    filtered_array_iter::FilteredArrayIter,
    items::inventory::InventoryState,
    structures::{
        structure_manifest::{Structure, StructureManifest},
        Footprint,
    },
    units::actions::DeliveryMode,
    water::WaterTable,
};

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
    /// Use [`TilePos::all_valid_neighbors`] to get only valid neighbors.
    #[inline]
    #[must_use]
    pub(crate) fn all_neighbors(&self) -> impl IntoIterator<Item = TilePos> {
        self.hex.all_neighbors().map(|hex| TilePos { hex })
    }

    /// All adjacent tiles that are on the map.
    #[inline]
    #[must_use]
    pub fn all_valid_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&pos| map_geometry.is_valid(pos));
        iter
    }

    /// All adjacent tiles that are at most [`Height::MAX_STEP`] higher or lower than `self`.
    ///
    /// If the adjacent tile contains a structure, the height of the structure is added to the tile height.
    #[inline]
    #[must_use]
    pub(crate) fn reachable_neighbors(
        &self,
        structure_query: &Query<&Id<Structure>>,
        structure_manifest: &StructureManifest,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        if !map_geometry.is_valid(*self) {
            let null_array = [TilePos::ZERO; 6];
            let mut null_iter = FilteredArrayIter::from(null_array);
            null_iter.filter(|_| false);
            return null_iter;
        }

        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        let self_height = map_geometry.get_height(*self).unwrap();

        iter.filter(|&target_pos| {
            if !map_geometry.is_valid(target_pos) {
                return false;
            }

            let terrain_height = map_geometry.get_height(target_pos).unwrap();

            if self_height > terrain_height {
                // PERF: oh god this is a lot of indirection. We should consider moving away from a pure manifest system
                let structure_height = if let Some(structure_entity) =
                    map_geometry.get_structure(target_pos)
                {
                    let structure_id = *structure_query.get(structure_entity).unwrap();
                    let structure_data = structure_manifest.get(structure_id);
                    structure_data.height
                } else if let Some(ghost_entity) = map_geometry.get_ghost_structure(target_pos) {
                    let structure_id = *structure_query.get(ghost_entity).unwrap();
                    let structure_data = structure_manifest.get(structure_id);
                    structure_data.height
                } else {
                    Height::ZERO
                };

                // If we are reaching down, we can take advantage of the height of the structure
                self_height - terrain_height <= structure_height + Height::MAX_STEP
            } else {
                // We don't care how tall the structure is if we are reaching up
                terrain_height - self_height <= Height::MAX_STEP
            }
        });
        iter
    }

    /// All adjacent tiles that are passable.
    ///
    /// This is distinct from [`reachable_neighbors`](Self::reachable_neighbors), which includes tiles filled with litter.
    #[inline]
    #[must_use]
    pub(crate) fn passable_neighbors(
        &self,
        map_geometry: &MapGeometry,
        water_table: &WaterTable,
    ) -> impl IntoIterator<Item = TilePos> {
        if !map_geometry.is_valid(*self) {
            let null_array = [TilePos::ZERO; 6];
            let mut null_iter = FilteredArrayIter::from(null_array);
            null_iter.filter(|_| false);
            return null_iter;
        }

        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&target_pos| {
            map_geometry.is_valid(target_pos)
                && map_geometry.is_passable(*self, target_pos, water_table)
        });
        iter
    }

    /// All adjacent tiles that are out of bounds.
    #[inline]
    #[must_use]
    pub(crate) fn out_of_bounds_neighbors(
        &self,
        map_geometry: &MapGeometry,
    ) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&target_pos| !map_geometry.is_valid(target_pos));
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

/// The overall size and arrangement of the map.
#[derive(Debug, Resource)]
pub struct MapGeometry {
    /// The size and orientation of the map.
    pub(crate) layout: HexLayout,
    /// The number of tiles from the center to the edge of the map.
    ///
    /// Note that the central tile is not counted.
    pub(crate) radius: u32,
    /// Which [`Terrain`](crate::terrain::terrain_manifest::Terrain) entity is stored at each tile position
    terrain_index: HashMap<TilePos, Entity>,
    /// Which [`Id<Structure>`](crate::asset_management::manifest::Id) entity is stored at each tile position
    structure_index: HashMap<TilePos, Entity>,
    /// Which [`Ghost`](crate::construction::ghosts::Ghost) structure entity is stored at each tile position
    ghost_structure_index: HashMap<TilePos, Entity>,
    /// Which [`Ghost`](crate::construction::ghosts::Ghost) terrain entity is stored at each tile position
    ghost_terrain_index: HashMap<TilePos, Entity>,
    /// The set of tiles that cannot be traversed by units due to structures.
    impassable_structure_tiles: HashSet<TilePos>,
    /// The set of tiles that cannot be traversed by units due to litter.
    impassable_litter_tiles: HashSet<TilePos>,
    /// The height of the terrain at each tile position.
    height_index: HashMap<TilePos, Height>,
    /// Tracks any current wormholes, and their destinations.
    ///
    /// The key is the tile position of the wormhole's entrance.
    /// The value is the tile position of the wormhole's exit.
    /// This map is symmetric, so the exit tile position is also a key.
    pub(crate) wormhole_index: HashMap<TilePos, TilePos>,
}

/// A [`MapGeometry`] index was missing an entry.
#[derive(Debug, PartialEq)]
pub struct IndexError {
    /// The tile position that was missing.
    pub tile_pos: TilePos,
}

impl MapGeometry {
    /// Creates a new [`MapGeometry`] of the provided raidus.
    ///
    /// All indexes will be empty.
    pub fn new(radius: u32) -> Self {
        let tiles = hexagon(Hex::ZERO, radius).map(|hex| TilePos { hex });
        // We can start with the minimum height everywhere as no entities need to be spawned.
        let height_index = tiles.map(|tile_pos| (tile_pos, Height::MIN)).collect();

        MapGeometry {
            layout: HexLayout::default(),
            radius,
            terrain_index: HashMap::default(),
            structure_index: HashMap::default(),
            ghost_structure_index: HashMap::default(),
            ghost_terrain_index: HashMap::default(),
            impassable_structure_tiles: HashSet::default(),
            impassable_litter_tiles: HashSet::default(),
            wormhole_index: HashMap::default(),
            height_index,
        }
    }

    /// Returns the list of valid tile positions.
    #[inline]
    pub fn valid_tile_positions(&self) -> impl ExactSizeIterator<Item = TilePos> + '_ {
        hexagon(Hex::ZERO, self.radius).map(|hex| TilePos { hex })
    }

    /// Is the provided `tile_pos` in the map?
    #[inline]
    #[must_use]
    pub(crate) fn is_valid(&self, tile_pos: TilePos) -> bool {
        let distance = Hex::ZERO.distance_to(tile_pos.hex);
        distance <= self.radius as i32
    }

    /// Are all of the tiles in the `footprint` centered around `center` valid?
    #[inline]
    #[must_use]
    pub(crate) fn is_footprint_valid(
        &self,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|tile_pos| self.is_valid(*tile_pos))
    }

    /// Is the provided `tile_pos` passable?
    ///
    /// Tiles that are not part of the map will return `false`.
    /// Tiles that have a structure will return `false`.
    /// Tiles that are more than [`Height::MAX_STEP`] above or below the current tile will return `false`.
    /// Tiles that are completely full of litter will return `false`.
    #[inline]
    #[must_use]
    pub(crate) fn is_passable(
        &self,
        starting_pos: TilePos,
        ending_pos: TilePos,
        water_table: &WaterTable,
    ) -> bool {
        if !self.is_valid(starting_pos) {
            return false;
        }

        if !self.is_valid(ending_pos) {
            return false;
        }

        if self.impassable_structure_tiles.contains(&ending_pos) {
            return false;
        }

        if self.impassable_litter_tiles.contains(&ending_pos) {
            return false;
        }

        if water_table.surface_water_depth(ending_pos) > Height::WADING_DEPTH {
            return false;
        }

        if let Ok(height_difference) = self.height_difference(starting_pos, ending_pos) {
            height_difference <= Height::MAX_STEP
        } else {
            false
        }
    }

    /// Is there enough space for a structure with the provided `footprint` located at the `center` tile?
    #[inline]
    #[must_use]
    pub(crate) fn is_space_available(
        &self,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|tile_pos| self.get_structure(*tile_pos).is_none())
    }

    /// Is there enough space for `existing_entity` to transform into a structure with the provided `footprint` located at the `center` tile?
    ///
    /// The `existing_entity` will be ignored when checking for space.
    #[inline]
    #[must_use]
    fn is_space_available_to_transform(
        &self,
        existing_entity: Entity,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint.normalized(facing, center).iter().all(|tile_pos| {
            let structure_entity = self.get_structure(*tile_pos);
            let ghost_structure_entity = self.get_ghost_structure(*tile_pos);

            (structure_entity.is_none() || structure_entity == Some(existing_entity))
                && ghost_structure_entity.is_none()
        })
    }

    /// Are all of the terrain tiles in the provided `footprint` flat?
    #[inline]
    #[must_use]
    pub(crate) fn is_terrain_flat(
        &self,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        let Some(height) = footprint.height(facing, center, self) else { return false };

        footprint
            .normalized(facing, center)
            .iter()
            .all(|tile_pos| self.get_height(*tile_pos) == Ok(height))
    }

    /// Can the structure with the provided `footprint` be built at the `center` tile?
    ///
    /// The provided [`Footprint`] *must* be rotated to the correct orientation,
    /// matching the [`Facing`] of the structure.
    ///
    /// This checks that:
    /// - the area is in the map
    /// - the area is flat
    /// - the area is free of structures
    /// - there is no surface water present
    #[inline]
    #[must_use]
    pub(crate) fn can_build(
        &self,
        center: TilePos,
        footprint: &Footprint,
        height: Height,
        facing: Facing,
        water_table: &WaterTable,
    ) -> bool {
        self.is_footprint_valid(center, footprint, facing)
            && self.is_terrain_flat(center, footprint, facing)
            && self.is_space_available(center, footprint, facing)
            && self.is_free_of_water(center, footprint, height, facing, water_table)
    }

    /// Can the `existing_entity` transform into a structure with the provided `footprint` at the `center` tile?
    ///
    /// The provided [`Footprint`] *must* be rotated to the correct orientation,
    /// matching the [`Facing`] of the structure.
    ///
    /// This checks that:
    /// - the area is in the map
    /// - the area is flat
    /// - the area is free of structures
    /// - all tiles match the provided allowable terrain list
    #[inline]
    #[must_use]
    pub(crate) fn can_transform(
        &self,
        existing_entity: Entity,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        self.is_footprint_valid(center, footprint, facing)
            && self.is_terrain_flat(center, footprint, facing)
            && self.is_space_available_to_transform(existing_entity, center, footprint, facing)
    }

    /// Updates the height of the tile at `tile_pos`
    #[inline]
    pub(crate) fn update_height(&mut self, tile_pos: TilePos, height: Height) {
        assert!(
            self.is_valid(tile_pos),
            "Invalid tile position: {:?} with a radius of {:?}",
            tile_pos,
            self.radius
        );
        assert!(height >= Height(0.));

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
    #[inline]
    #[must_use]
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

    /// Returns the absolute difference in height between the tile at `starting_pos` and the tile at `ending_pos`.
    #[inline]
    pub(crate) fn height_difference(
        &self,
        starting_pos: TilePos,
        ending_pos: TilePos,
    ) -> Result<Height, IndexError> {
        let starting_height = self.get_height(starting_pos)?;
        let ending_height = self.get_height(ending_pos)?;
        Ok(starting_height.abs_diff(ending_height))
    }

    /// Flattens the terrain in the `footprint` around `tile_pos` to the height at that location.
    ///
    /// This footprint is rotated by the supplied `facing`.
    pub(crate) fn flatten_height(
        &mut self,
        height_query: &mut Query<&mut Height>,
        center: TilePos,
        footprint: &Footprint,
        facing: Facing,
    ) {
        let Ok(target_height) = self.get_height(center) else { return };
        for tile_pos in footprint.normalized(facing, center) {
            if let Some(entity) = self.get_terrain(tile_pos) {
                if let Ok(mut height) = height_query.get_mut(entity) {
                    *height = target_height;
                    self.update_height(tile_pos, target_height);
                }
            }
        }
    }

    /// Gets the [`Entity`] at the provided `tile_pos` that might have or want an item.
    ///
    /// If the `delivery_mode` is [`DeliveryMode::PickUp`], looks for litter, ghost terrain, or structures.
    /// If the `delivery_mode` is [`DeliveryMode::DropOff`], looks for ghost structures, ghost terrain or structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_candidates(
        &self,
        tile_pos: TilePos,
        delivery_mode: DeliveryMode,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();

        match delivery_mode {
            DeliveryMode::DropOff => {
                if let Some(&structure_entity) = self.structure_index.get(&tile_pos) {
                    entities.push(structure_entity)
                }

                if let Some(&ghost_terrain_entity) = self.ghost_terrain_index.get(&tile_pos) {
                    entities.push(ghost_terrain_entity)
                }

                if let Some(&ghost_structure_entity) = self.ghost_structure_index.get(&tile_pos) {
                    entities.push(ghost_structure_entity)
                }
            }
            DeliveryMode::PickUp => {
                if let Some(&structure_entity) = self.structure_index.get(&tile_pos) {
                    entities.push(structure_entity)
                }

                if let Some(&ghost_terrain_entity) = self.ghost_terrain_index.get(&tile_pos) {
                    entities.push(ghost_terrain_entity)
                }

                if let Some(&litter_entity) = self.terrain_index.get(&tile_pos) {
                    entities.push(litter_entity)
                }
            }
        }

        entities
    }

    /// Gets entities that units might work at, at the provided `tile_pos`.
    ///
    /// Prioritizes ghosts over structures if both are present to allow for replacing structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_workplaces(&self, tile_pos: TilePos) -> Vec<Entity> {
        let mut entities = Vec::new();

        if let Some(&ghost_structure_entity) = self.ghost_structure_index.get(&tile_pos) {
            entities.push(ghost_structure_entity)
        }

        if let Some(&structure_entity) = self.structure_index.get(&tile_pos) {
            entities.push(structure_entity)
        }

        entities
    }

    /// Gets the terrain [`Entity`] at the provided `tile_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_terrain(&self, tile_pos: TilePos) -> Option<Entity> {
        self.terrain_index.get(&tile_pos).copied()
    }

    /// Adds the provided `terrain_entity` to the terrain index at the provided `tile_pos`.
    #[inline]
    pub(crate) fn add_terrain(&mut self, tile_pos: TilePos, terrain_entity: Entity) {
        self.terrain_index.insert(tile_pos, terrain_entity);
    }

    /// Gets the structure [`Entity`] at the provided `tile_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_structure(&self, tile_pos: TilePos) -> Option<Entity> {
        self.structure_index.get(&tile_pos).copied()
    }

    /// Adds the provided `structure_entity` to the structure index at the provided `center`.
    #[inline]
    pub(crate) fn add_structure(
        &mut self,
        facing: Facing,
        center: TilePos,
        footprint: &Footprint,
        passable: bool,
        structure_entity: Entity,
    ) {
        for tile_pos in footprint.normalized(facing, center) {
            self.structure_index.insert(tile_pos, structure_entity);
            if !passable {
                self.impassable_structure_tiles.insert(tile_pos);
            }
        }
    }

    /// Removes any structure entity found at the provided `tile_pos` from the structure index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_structure(&mut self, tile_pos: TilePos) -> Option<Entity> {
        let removed = self.structure_index.remove(&tile_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.structure_index.retain(|_k, v| *v != removed_entity);
            self.impassable_structure_tiles.remove(&tile_pos);
        };

        removed
    }

    /// Gets the ghost structure [`Entity`] at the provided `tile_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_ghost_structure(&self, tile_pos: TilePos) -> Option<Entity> {
        self.ghost_structure_index.get(&tile_pos).copied()
    }

    /// Adds the provided `ghost_structure_entity` to the ghost structure index at the provided `center`.
    #[inline]
    pub(crate) fn add_ghost_structure(
        &mut self,
        facing: Facing,
        center: TilePos,
        footprint: &Footprint,
        ghost_structure_entity: Entity,
    ) {
        for tile_pos in footprint.normalized(facing, center) {
            self.ghost_structure_index
                .insert(tile_pos, ghost_structure_entity);
        }
    }

    /// Removes any ghost structure entity found at the provided `tile_pos` from the ghost structure index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_ghost_structure(&mut self, tile_pos: TilePos) -> Option<Entity> {
        let removed = self.ghost_structure_index.remove(&tile_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.ghost_structure_index
                .retain(|_k, v| *v != removed_entity);
        };

        removed
    }

    /// Adds the provided `ghost_terrain_entity` to the ghost terrain index at the provided `tile_pos`.
    #[inline]
    pub(crate) fn add_ghost_terrain(&mut self, ghost_terrain_entity: Entity, tile_pos: TilePos) {
        self.ghost_terrain_index
            .insert(tile_pos, ghost_terrain_entity);
    }

    /// Removes any ghost terrain entity found at the provided `tile_pos` from the ghost terrain index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_ghost_terrain(&mut self, tile_pos: TilePos) -> Option<Entity> {
        let removed = self.ghost_terrain_index.remove(&tile_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.ghost_terrain_index
                .retain(|_k, v| *v != removed_entity);
        };

        removed
    }

    /// Gets the ghost terrain [`Entity`] at the provided `tile_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_ghost_terrain(&self, tile_pos: TilePos) -> Option<Entity> {
        self.ghost_terrain_index.get(&tile_pos).copied()
    }

    /// Updates the passability of the provided `tile_pos` based on the state of the litter at that location.
    pub(crate) fn update_litter_state(&mut self, tile_pos: TilePos, litter_state: InventoryState) {
        match litter_state {
            InventoryState::Empty | InventoryState::Partial => {
                self.impassable_litter_tiles.remove(&tile_pos);
            }
            InventoryState::Full => {
                self.impassable_litter_tiles.insert(tile_pos);
            }
        }
    }

    /// Are all of the tiles defined by `footprint` located at the `center` tile free of surface water?
    #[inline]
    #[must_use]
    pub(crate) fn is_free_of_water(
        &self,
        center: TilePos,
        footprint: &Footprint,
        height: Height,
        facing: Facing,
        water_table: &WaterTable,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|tile_pos| water_table.surface_water_depth(*tile_pos) <= height)
    }

    /// Returns an iterator over all of the tiles that are ocean tiles.
    #[inline]
    #[must_use]
    pub(crate) fn ocean_tiles(&self) -> impl ExactSizeIterator<Item = TilePos> + '_ {
        // Oceans ring the entire map currently
        let hex_ring = Hex::ZERO.ring(self.radius + 1);
        hex_ring.map(move |hex| TilePos { hex })
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
    /// Generates a random facing.
    #[inline]
    #[must_use]
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        let direction = *Direction::ALL_DIRECTIONS.choose(rng).unwrap();

        Self { direction }
    }

    /// Rotates this facing one 60 degree step counterclockwise.
    #[inline]
    pub(crate) fn rotate_counterclockwise(&mut self) {
        self.direction = self.direction.counter_clockwise();
    }

    /// Rotates this facing one 60 degree step clockwise.
    #[inline]
    pub(crate) fn rotate_clockwise(&mut self) {
        self.direction = self.direction.clockwise();
    }

    /// Returns the number of clockwise 60 degree rotations needed to face this direction, starting from [`Direction::Top`].
    ///
    /// This is intended to be paired with [`Hex::rotate_clockwise`](hexx::Hex) to rotate a hex to face this direction.
    #[inline]
    pub(crate) const fn rotation_count(&self) -> u32 {
        match self.direction {
            Direction::Top => 0,
            Direction::TopLeft => 1,
            Direction::BottomLeft => 2,
            Direction::Bottom => 3,
            Direction::BottomRight => 4,
            Direction::TopRight => 5,
        }
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
    #[inline]
    #[must_use]
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen::<bool>() {
            true => RotationDirection::Left,
            false => RotationDirection::Right,
        }
    }
}

// BLOCKED: manual implementation of https://github.com/ManevilleF/hexx/issues/84
// PERF: this is terrible, use a lookup table
/// Converts an angle in radians to a [`Direction`].
pub(crate) fn direction_from_angle(radians: f32, orientation: HexOrientation) -> Direction {
    // Clamp to [0, 2π)
    let radians = radians.rem_euclid(TAU);

    let direction_angle_pairs = Direction::ALL_DIRECTIONS.map(|direction| {
        let angle = direction.angle(&orientation).rem_euclid(TAU);
        (direction, angle)
    });

    let mut current_best_direction = Direction::Top;
    let mut current_best_delta = f32::MAX;

    let mut lowest_direction = Direction::Top;
    let mut lowest_angle = f32::MAX;

    let mut highest_direction = Direction::Top;
    let mut highest_angle = 0.;

    for (direction, angle) in direction_angle_pairs {
        if angle > highest_angle {
            highest_direction = direction;
            highest_angle = angle;
        }

        if angle < lowest_angle {
            lowest_direction = direction;
            lowest_angle = angle;
        }

        let delta = (angle - radians).abs();
        if delta < current_best_delta {
            current_best_direction = direction;
            current_best_delta = delta;
        }
    }

    // Handle the case where the angle is between the highest and lowest angles
    if radians > highest_angle {
        let lowest_angle = lowest_angle + TAU;
        if (radians - highest_angle).abs() < (radians - lowest_angle).abs() {
            highest_direction
        } else {
            lowest_direction
        }
    } else {
        current_best_direction
    }
}

/// Constructs the mesh for a single hexagonal column with the specified height.
#[must_use]
pub(crate) fn hexagonal_column(hex_layout: &HexLayout, hex_height: f32) -> Mesh {
    let mesh_info = ColumnMeshBuilder::new(hex_layout, hex_height).build();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs.to_vec());
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
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

    #[test]
    fn adding_multi_tile_structure_adds_to_index() {
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let center = TilePos::new(17, -2);
        let passable = false;

        map_geometry.add_structure(facing, center, &footprint, passable, structure_entity);

        // Check that the structure index was updated correctly
        for tile_pos in footprint.normalized(facing, center) {
            assert_eq!(Some(structure_entity), map_geometry.get_structure(tile_pos));
        }
    }

    #[test]
    fn removing_multi_tile_structure_clears_indexes() {
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let center = TilePos::new(17, -2);
        let passable = false;

        map_geometry.add_structure(facing, center, &footprint, passable, structure_entity);
        map_geometry.remove_structure(center);

        // Check that the structure index was updated correctly
        for tile_pos in footprint.normalized(facing, center) {
            dbg!(tile_pos);
            assert_eq!(None, map_geometry.get_structure(tile_pos));
        }
    }

    #[test]
    fn direction_from_angle_works_for_exact_values() {
        for direction in Direction::ALL_DIRECTIONS {
            let pointy_radians = direction.angle(&HexOrientation::pointy());
            let pointy_direction = direction_from_angle(pointy_radians, HexOrientation::pointy());

            let flat_radians = direction.angle(&HexOrientation::flat());
            let flat_direction = direction_from_angle(flat_radians, HexOrientation::flat());

            assert_eq!(
                direction, pointy_direction,
                "Failed for {:?}",
                pointy_radians
            );
            assert_eq!(direction, flat_direction, "Failed for {:?}", flat_radians);
        }
    }

    #[test]
    fn direction_from_angle_works_for_large_and_small_values() {
        for direction in Direction::ALL_DIRECTIONS {
            let radians = direction.angle(&HexOrientation::flat());

            let large_radians = radians + TAU;
            let small_radians = radians - TAU;

            let large_direction = direction_from_angle(large_radians, HexOrientation::flat());
            let small_direction = direction_from_angle(small_radians, HexOrientation::flat());

            assert_eq!(direction, large_direction, "Failed for {:?}", large_radians);
            assert_eq!(direction, small_direction, "Failed for {:?}", small_radians);
        }
    }

    #[test]
    fn direction_from_angle_works_with_small_offsets() {
        let epsilon = 0.001;
        assert!(epsilon < TAU / 12.);

        for direction in Direction::ALL_DIRECTIONS {
            let radians = direction.angle(&HexOrientation::flat());

            let large_radians = radians + epsilon;
            let small_radians = radians - epsilon;

            let large_direction = direction_from_angle(large_radians, HexOrientation::flat());
            let small_direction = direction_from_angle(small_radians, HexOrientation::flat());

            assert_eq!(direction, large_direction, "Failed for {:?}", large_radians);
            assert_eq!(direction, small_direction, "Failed for {:?}", small_radians);
        }
    }
}
