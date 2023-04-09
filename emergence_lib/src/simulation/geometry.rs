//! Manages the game world's grid and data tied to that grid

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
};
use core::fmt::Display;
use derive_more::{Add, AddAssign, Display, Sub, SubAssign};
use hexx::{shapes::hexagon, Direction, Hex, HexLayout, MeshInfo};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    asset_management::manifest::Id, construction::AllowedTerrainTypes,
    filtered_array_iter::FilteredArrayIter, items::inventory::InventoryState,
    structures::Footprint, terrain::terrain_manifest::Terrain, units::actions::DeliveryMode,
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
    pub fn all_neighbors(&self, map_geometry: &MapGeometry) -> impl IntoIterator<Item = TilePos> {
        let neighbors = self.hex.all_neighbors().map(|hex| TilePos { hex });
        let mut iter = FilteredArrayIter::from(neighbors);
        iter.filter(|&pos| map_geometry.is_valid(pos));
        iter
    }

    /// All adjacent tiles that are at most [`Height::MAX_STEP`] higher or lower than `self`.
    pub(crate) fn reachable_neighbors(
        &self,
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
        iter.filter(|&target_pos| {
            map_geometry.is_valid(target_pos)
                && map_geometry.height_difference(*self, target_pos).unwrap() <= Height::MAX_STEP
        });
        iter
    }

    /// All adjacent tiles that are passable.
    ///
    /// This is distinct from [`reachable_neighbors`](Self::reachable_neighbors), which includes tiles filled with litter.
    pub(crate) fn passable_neighbors(
        &self,
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
        iter.filter(|&target_pos| {
            map_geometry.is_valid(target_pos) && map_geometry.is_passable(*self, target_pos)
        });
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

    /// Returns the [`TilePos`] rotated to match the `facing` around the origin.
    pub(crate) fn rotated(&self, facing: Facing) -> Self {
        let n_rotations = facing.rotation_count();

        TilePos {
            hex: self.hex.rotate_right(n_rotations),
        }
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

    /// The maximum height difference that units can traverse in a single step.
    pub(crate) const MAX_STEP: Height = Height(1);

    /// The thickness of all terrain topper models.
    /// Note that the diameter of a tile is 1.0 transform units.
    pub(crate) const TOPPER_THICKNESS: f32 = 0.224;

    /// The height of each step up, in world coordinates.
    pub(crate) const STEP_HEIGHT: f32 = 1.0;

    /// Computes the `y` coordinate of a `Transform` that corresponds to this height.
    pub(crate) fn into_world_pos(self) -> f32 {
        self.0 as f32 * Self::STEP_HEIGHT
    }

    /// Constructs a new height from the `y` coordinate of a `Transform`.
    ///
    /// Any values outside of the allowable range will be clamped to [`Height::MIN`] and [`Height::MAX`] appropriately.
    pub(crate) fn from_world_pos(world_y: f32) -> Self {
        let f32_height = (world_y / Self::STEP_HEIGHT).round();
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

impl Add for Height {
    type Output = Height;

    fn add(self, rhs: Self) -> Self::Output {
        Height(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Height {
    type Output = Height;

    fn sub(self, rhs: Self) -> Self::Output {
        Height(self.0.saturating_sub(rhs.0))
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
    /// The amount of litter at each tile position
    litter_index: HashMap<TilePos, InventoryState>,
    /// The height of the terrain at each tile position
    height_index: HashMap<TilePos, Height>,
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
            litter_index: HashMap::default(),
            height_index,
        }
    }

    /// Is the provided `tile_pos` in the map?
    pub(crate) fn is_valid(&self, tile_pos: TilePos) -> bool {
        let distance = Hex::ZERO.distance_to(tile_pos.hex);
        distance <= self.radius as i32
    }

    /// Are all of the tiles in the `footprint` centered around `center` in the map?
    pub(crate) fn is_footprint_valid(&self, tile_pos: TilePos, footprint: &Footprint) -> bool {
        footprint
            .in_world_space(tile_pos)
            .iter()
            .all(|tile_pos| self.is_valid(*tile_pos))
    }

    /// Is the provided `tile_pos` passable?
    ///
    /// Tiles that are not part of the map will return `false`.
    /// Tiles that have a structure will return `false`.
    /// Tiles that are more than [`Height::MAX_STEP`] above or below the current tile will return `false`.
    /// Tiles that are completely full of litter will return `false`.
    pub(crate) fn is_passable(&self, starting_pos: TilePos, ending_pos: TilePos) -> bool {
        if !self.is_valid(starting_pos) {
            return false;
        }

        if !self.is_valid(ending_pos) {
            return false;
        }

        if self.get_structure(ending_pos).is_some() {
            return false;
        }

        if self.get_litter_state(ending_pos) == InventoryState::Full {
            return false;
        }

        if let Ok(height_difference) = self.height_difference(starting_pos, ending_pos) {
            height_difference <= Height::MAX_STEP
        } else {
            false
        }
    }

    /// Is there enough space for a structure with the provided `footprint` located at the `center` tile?
    fn is_space_available(&self, center: TilePos, footprint: &Footprint) -> bool {
        footprint
            .in_world_space(center)
            .iter()
            .all(|tile_pos| self.get_structure(*tile_pos).is_none())
    }

    /// Are all of the terrain tiles in the provided `footprint` appropriate?
    fn is_terrain_valid(
        &self,
        center: TilePos,
        footprint: &Footprint,
        terrain_query: &Query<&Id<Terrain>>,
        allowed_terrain_types: &AllowedTerrainTypes,
    ) -> bool {
        match allowed_terrain_types {
            AllowedTerrainTypes::Any => true,
            AllowedTerrainTypes::Only(allowed_terrain_types) => {
                footprint.in_world_space(center).iter().all(|tile_pos| {
                    let terrain_entity = self.terrain_index.get(tile_pos).unwrap();
                    let terrain_id = terrain_query.get(*terrain_entity).unwrap();
                    allowed_terrain_types.contains(terrain_id)
                })
            }
        }
    }

    /// Are all of the terrain tiles in the provided `footprint` flat?
    fn is_terrain_flat(&self, center: TilePos, footprint: &Footprint) -> bool {
        let height = self.get_height(center).unwrap();

        footprint
            .in_world_space(center)
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
    /// - all tiles match the provided allowable terrain list
    pub(crate) fn can_build(
        &self,
        center: TilePos,
        footprint: Footprint,
        terrain_query: &Query<&Id<Terrain>>,
        allowed_terrain_types: &AllowedTerrainTypes,
    ) -> bool {
        self.is_footprint_valid(center, &footprint)
            && self.is_terrain_flat(center, &footprint)
            && self.is_space_available(center, &footprint)
            && self.is_terrain_valid(center, &footprint, terrain_query, allowed_terrain_types)
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

    /// Returns the absolute difference in height between the tile at `starting_pos` and the tile at `ending_pos`.
    pub(crate) fn height_difference(
        &self,
        starting_pos: TilePos,
        ending_pos: TilePos,
    ) -> Result<Height, IndexError> {
        let starting_height = self.get_height(starting_pos)?;
        let ending_height = self.get_height(ending_pos)?;
        Ok(Height(starting_height.abs_diff(ending_height.0)))
    }

    /// Gets the [`Entity`] at the provided `tile_pos` that might have or want an item.
    ///
    /// If the `delivery_mode` is [`DeliveryMode::PickUp`], looks for litter, ghost terrain, or structures.
    /// If the `delivery_mode` is [`DeliveryMode::DropOff`], looks for ghost structures, ghost terrain or structures.
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
    pub(crate) fn get_terrain(&self, tile_pos: TilePos) -> Option<Entity> {
        self.terrain_index.get(&tile_pos).copied()
    }

    /// Adds the provided `terrain_entity` to the terrain index at the provided `tile_pos`.
    pub(crate) fn add_terrain(&mut self, tile_pos: TilePos, terrain_entity: Entity) {
        self.terrain_index.insert(tile_pos, terrain_entity);
    }

    /// Gets the structure [`Entity`] at the provided `tile_pos`, if any.
    pub(crate) fn get_structure(&self, tile_pos: TilePos) -> Option<Entity> {
        self.structure_index.get(&tile_pos).copied()
    }

    /// Adds the provided `structure_entity` to the structure index at the provided `center`.
    pub(crate) fn add_structure(
        &mut self,
        center: TilePos,
        footprint: &Footprint,
        structure_entity: Entity,
    ) {
        for tile_pos in footprint.in_world_space(center) {
            self.structure_index.insert(tile_pos, structure_entity);
        }
    }

    /// Removes any structure entity found at the provided `tile_pos` from the structure index.
    ///
    /// Returns the removed entity, if any.
    pub(crate) fn remove_structure(&mut self, tile_pos: TilePos) -> Option<Entity> {
        let removed = self.structure_index.remove(&tile_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.structure_index.retain(|_k, v| *v != removed_entity);
        };

        removed
    }

    /// Gets the ghost structure [`Entity`] at the provided `tile_pos`, if any.
    pub(crate) fn get_ghost_structure(&self, tile_pos: TilePos) -> Option<Entity> {
        self.ghost_structure_index.get(&tile_pos).copied()
    }

    /// Adds the provided `ghost_structure_entity` to the ghost structure index at the provided `center`.
    pub(crate) fn add_ghost_structure(
        &mut self,
        center: TilePos,
        footprint: &Footprint,
        ghost_structure_entity: Entity,
    ) {
        for tile_pos in footprint.in_world_space(center) {
            self.ghost_structure_index
                .insert(tile_pos, ghost_structure_entity);
        }
    }

    /// Removes any ghost structure entity found at the provided `tile_pos` from the ghost structure index.
    ///
    /// Returns the removed entity, if any.
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
    pub(crate) fn add_ghost_terrain(&mut self, ghost_terrain_entity: Entity, tile_pos: TilePos) {
        self.ghost_terrain_index
            .insert(tile_pos, ghost_terrain_entity);
    }

    /// Removes any ghost terrain entity found at the provided `tile_pos` from the ghost terrain index.
    ///
    /// Returns the removed entity, if any.
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
    pub(crate) fn get_ghost_terrain(&self, tile_pos: TilePos) -> Option<Entity> {
        self.ghost_terrain_index.get(&tile_pos).copied()
    }

    /// Sets the amount of litter at the provided `tile_pos`.
    pub(crate) fn set_litter_state(&mut self, tile_pos: TilePos, litter_state: InventoryState) {
        self.litter_index.insert(tile_pos, litter_state);
    }

    /// Gets the amount of litter at the provided `tile_pos`.
    pub(crate) fn get_litter_state(&self, tile_pos: TilePos) -> InventoryState {
        self.litter_index
            .get(&tile_pos)
            .copied()
            .unwrap_or(InventoryState::Empty)
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

    /// Returns the number of clockwise 60 degree rotations needed to face this direction, starting from [`Direction::Top`].
    ///
    /// This is intended to be paired with [`Hex::rotate_right`](hexx::Hex) to rotate a hex to face this direction.
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
    pub(crate) fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen::<bool>() {
            true => RotationDirection::Left,
            false => RotationDirection::Right,
        }
    }
}

/// Constructs the mesh for a single hexagonal column with the specified height.
pub(crate) fn hexagonal_column(hex_layout: &HexLayout, hex_height: f32) -> Mesh {
    let mesh_info = MeshInfo::hexagonal_column(hex_layout, Hex::ZERO, hex_height);
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

    #[test]
    fn adding_multi_tile_structure_adds_to_index() {
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let center = TilePos::new(17, -2);
        map_geometry.add_structure(center, &footprint, structure_entity);

        // Check that the structure index was updated correctly
        for tile_pos in footprint.in_world_space(center) {
            assert_eq!(Some(structure_entity), map_geometry.get_structure(tile_pos));
        }
    }

    #[test]
    fn removing_multi_tile_structure_clears_indexes() {
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let center = TilePos::new(17, -2);
        map_geometry.add_structure(center, &footprint, structure_entity);
        map_geometry.remove_structure(center);

        // Check that the structure index was updated correctly
        for tile_pos in footprint.in_world_space(center) {
            dbg!(tile_pos);
            assert_eq!(None, map_geometry.get_structure(tile_pos));
        }
    }
}
