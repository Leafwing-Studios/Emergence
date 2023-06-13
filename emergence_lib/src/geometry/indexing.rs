//! Tracks the location of key entities on the map, and caches information about the map for faster access.

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use hexx::{shapes::hexagon, Hex, HexLayout};

use crate::{
    items::inventory::InventoryState, structures::Footprint, units::actions::DeliveryMode,
};

use super::{DiscreteHeight, Facing, Height, VoxelKind, VoxelObject, VoxelPos};

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
    ///
    /// The set of keys is the set of all valid [`Hex`] positions on the map.
    terrain_index: HashMap<Hex, Entity>,
    /// The height of the terrain at each tile position.
    ///
    /// The set of keys is the set of all valid [`Hex`] positions on the map.
    height_index: HashMap<Hex, DiscreteHeight>,
    /// Tracks which objects are stored in each voxel.
    ///
    /// The set of keys is the set of all non-empty [`VoxelPos`] positions on the map.
    voxel_index: HashMap<VoxelPos, VoxelObject>,
    /// The list of all passable neighbors for each tile position.
    ///
    /// The set of keys is the set of all [`VoxelPos`] that units could be found.
    walkable_neighbors: HashMap<VoxelPos, [Option<VoxelPos>; 6]>,
    /// An empty list of walkable neighbors.
    empty_neighbors: [Option<VoxelPos>; 6],
}

/// A [`MapGeometry`] index was missing an entry.
#[derive(Debug, PartialEq)]
pub struct IndexError {
    /// The hex where the data was missing.
    pub hex: Hex,
}

/// An object could not be added to the [`MapGeometry`].
#[derive(Debug, PartialEq)]
pub enum AdditionError {
    /// An incompatible object was already present.
    AlreadyOccupied,
}

impl MapGeometry {
    /// Creates a new [`MapGeometry`] of the provided raidus.
    ///
    /// All indexes will be empty.
    pub fn new(world: &mut World, radius: u32) -> Self {
        let hexes: Vec<Hex> = hexagon(Hex::ZERO, radius).collect();
        let tiles: Vec<VoxelPos> = hexes
            .iter()
            .map(|hex| VoxelPos {
                hex: *hex,
                height: DiscreteHeight::ZERO,
            })
            .collect();

        // We can start with the minimum height everywhere as no entities need to be spawned.
        let height_index = tiles
            .iter()
            .map(|voxel_pos| (voxel_pos.hex, DiscreteHeight::ZERO))
            .collect();

        let mut terrain_index = HashMap::default();
        let mut voxel_index = HashMap::default();

        for hex in hexes {
            let voxel_pos = VoxelPos {
                hex,
                height: DiscreteHeight::ZERO,
            };
            // The TerrainPrototype component is used to track the terrain entities that need to be replaced with a full TerrainBundle
            let entity = world.spawn(voxel_pos).id();
            terrain_index.insert(hex, entity);
            voxel_index.insert(
                voxel_pos,
                VoxelObject {
                    entity,
                    object_kind: VoxelKind::Terrain,
                },
            );
        }

        let mut map_geometry = MapGeometry {
            layout: HexLayout::default(),
            radius,
            terrain_index,
            height_index,
            voxel_index,
            walkable_neighbors: HashMap::default(),
            empty_neighbors: [None; 6],
        };

        map_geometry.recompute_walkable_neighbors();

        #[cfg(test)]
        map_geometry.validate();

        map_geometry
    }

    /// Returns the list of all valid [`Hex`] positions on the map.
    #[inline]
    #[must_use]
    pub fn all_hexes(&self) -> impl Iterator<Item = &Hex> {
        self.terrain_index.keys()
    }

    /// Returns an iterator over all non-empty [`VoxelPos`] on the map.
    pub fn all_voxels(&self) -> impl Iterator<Item = (&VoxelPos, &VoxelObject)> {
        self.voxel_index.iter()
    }

    /// Is the provided `hex` in the map?
    #[inline]
    #[must_use]
    pub(crate) fn is_valid(&self, hex: Hex) -> bool {
        let distance = Hex::ZERO.distance_to(hex);
        distance <= self.radius as i32
    }

    /// Gets the voxel object at the provided `voxel_pos`.
    #[inline]
    #[must_use]
    pub(crate) fn get_voxel(&self, voxel_pos: VoxelPos) -> Option<&VoxelObject> {
        self.voxel_index.get(&voxel_pos)
    }

    /// Returns the voxel position directly above the terrain at `hex`
    #[inline]
    #[must_use]
    pub(crate) fn on_top_of_terrain(&self, hex: Hex) -> VoxelPos {
        let terrain_height: DiscreteHeight = self.get_height(hex).unwrap_or_default().into();
        VoxelPos {
            hex,
            height: terrain_height.above(),
        }
    }

    /// Are all of the tiles in the `footprint` centered around `center` valid?
    #[inline]
    #[must_use]
    pub(crate) fn is_footprint_valid(
        &self,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|voxel_pos| self.is_valid(voxel_pos.hex))
    }

    /// Is the provided `voxel_pos` passable?
    ///
    /// Tiles that are not part of the map will return `false`.
    /// Tiles that have a structure will return `false`.
    /// Tiles that are more than [`Height::MAX_STEP`] above or below the current tile will return `false`.
    /// Tiles that are completely full of litter will return `false`.
    #[inline]
    #[must_use]
    pub(crate) fn is_passable(&self, starting_pos: VoxelPos, ending_pos: VoxelPos) -> bool {
        if !self.is_valid(starting_pos.hex) {
            return false;
        }

        if !self.is_valid(ending_pos.hex) {
            return false;
        }

        if let Some(voxel_data) = self.get_voxel(starting_pos) {
            if !voxel_data.object_kind.can_walk_through() {
                return false;
            }
        }

        starting_pos.abs_height_diff(ending_pos) <= Height::MAX_STEP
    }

    /// Is there enough space for a structure with the provided `footprint` located at the `center` tile?
    #[inline]
    #[must_use]
    pub(crate) fn is_space_available(
        &self,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> Result<(), AdditionError> {
        match footprint
            .normalized(facing, center)
            .iter()
            .all(|voxel_pos| self.is_voxel_clear(*voxel_pos).is_ok())
        {
            true => Ok(()),
            false => Err(AdditionError::AlreadyOccupied),
        }
    }

    /// Is there space in a single voxel?
    #[inline]
    #[must_use]
    fn is_voxel_clear(&self, voxel_pos: VoxelPos) -> Result<(), AdditionError> {
        match self.voxel_index.contains_key(&voxel_pos) {
            true => Err(AdditionError::AlreadyOccupied),
            false => Ok(()),
        }
    }

    /// Is there enough space for `existing_entity` to transform into a structure with the provided `footprint` located at the `center` tile?
    ///
    /// The `existing_entity` will be ignored when checking for space.
    #[inline]
    #[must_use]
    fn is_space_available_to_transform(
        &self,
        existing_entity: Entity,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|voxel_pos| {
                let structure_entity = self.get_structure(*voxel_pos);
                let ghost_structure_entity = self.get_ghost_structure(*voxel_pos);

                (structure_entity.is_none() || structure_entity == Some(existing_entity))
                    && ghost_structure_entity.is_none()
            })
    }

    /// Are all of the terrain tiles in the provided `footprint` flat?
    // FIXME: this is the wrong check: we want to check that the terrain can fit the voxels, not that it's flat
    #[inline]
    #[must_use]
    pub(crate) fn is_terrain_flat(
        &self,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        let Some(height) = footprint.height(facing, center, self) else { return false };

        footprint
            .normalized(facing, center)
            .iter()
            .all(|voxel_pos| self.get_height(voxel_pos.hex) == Ok(height))
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
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        self.is_footprint_valid(center, footprint, facing)
            && self.is_terrain_flat(center, footprint, facing)
            && self.is_space_available_to_transform(existing_entity, center, footprint, facing)
    }

    /// Returns the height of the tile at `voxel_pos`, if available.
    ///
    /// This should always be [`Ok`] for all valid tiles.
    pub(crate) fn get_height(&self, hex: Hex) -> Result<DiscreteHeight, IndexError> {
        match self.height_index.get(&hex) {
            Some(height) => Ok(*height),
            None => Err(IndexError { hex }),
        }
    }

    /// Returns the average height (in world units) of tiles around `voxel_pos` within `radius`
    #[inline]
    #[must_use]
    pub(crate) fn average_height(&self, voxel_pos: VoxelPos, radius: u32) -> f32 {
        let hex_iter = hexagon(voxel_pos.hex, radius);
        let heights = hex_iter.filter(|hex| self.is_valid(*hex)).map(|hex| {
            let height = self.get_height(hex).unwrap();
            height.into_world_pos()
        });
        let n = Hex::range_count(radius);
        heights.sum::<f32>() / n as f32
    }

    /// Gets the [`Entity`] at the provided `voxel_pos` that might have or want an item.
    ///
    /// If the `delivery_mode` is [`DeliveryMode::PickUp`], looks for litter, ghost terrain, or structures.
    /// If the `delivery_mode` is [`DeliveryMode::DropOff`], looks for ghost structures, ghost terrain or structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_candidate(
        &self,
        voxel_pos: VoxelPos,
        delivery_mode: DeliveryMode,
    ) -> Option<Entity> {
        if let Some(voxel_data) = self.get_voxel(voxel_pos) {
            match delivery_mode {
                DeliveryMode::DropOff => {
                    if voxel_data.object_kind.can_drop_off() {
                        Some(voxel_data.entity)
                    } else {
                        None
                    }
                }
                DeliveryMode::PickUp => {
                    if voxel_data.object_kind.can_pick_up() {
                        Some(voxel_data.entity)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    /// Gets entities that units might work at, at the provided `voxel_pos`.
    ///
    /// Prioritizes ghosts over structures if both are present to allow for replacing structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_workplace(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        if let Some(voxel_data) = self.get_voxel(voxel_pos) {
            if voxel_data.object_kind.can_work_at() {
                Some(voxel_data.entity)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets the terrain [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub fn get_terrain(&self, hex: Hex) -> Result<Entity, IndexError> {
        match self.terrain_index.get(&hex).copied() {
            Some(entity) => Ok(entity),
            None => Err(IndexError { hex }),
        }
    }

    /// Updates the [`Height`] of the terrain at the provided `hex` to `height`.
    #[inline]
    pub fn update_height(&mut self, hex: Hex, height: DiscreteHeight) {
        let old_height = self.get_height(hex).unwrap().into();
        if old_height == height {
            return;
        }

        let old_voxel_pos = VoxelPos {
            hex,
            height: old_height,
        };
        let new_voxel_pos = VoxelPos { hex, height };

        // This overwrites the existing entry
        self.height_index.insert(hex, height.into());
        // The old voxel needs to be removed, rather than overwritten, as it may be at a different height.
        self.voxel_index.remove(&old_voxel_pos);
        self.voxel_index.insert(
            new_voxel_pos,
            VoxelObject {
                entity: self.get_terrain(hex).unwrap(),
                object_kind: VoxelKind::Terrain,
            },
        );

        self.recompute_walkable_neighbors();

        #[cfg(test)]
        self.validate();
    }

    /// Gets the structure [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_structure(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        let voxel_data = self.get_voxel(voxel_pos)?;
        match voxel_data.object_kind {
            VoxelKind::Structure { .. } => Some(voxel_data.entity),
            _ => None,
        }
    }

    /// Adds the provided `structure_entity` to the voxel index at the provided `center`.
    #[inline]
    pub(crate) fn add_structure(
        &mut self,
        center: VoxelPos,
        facing: Facing,
        footprint: &Footprint,
        can_walk_on_roof: bool,
        can_walk_through: bool,
        structure_entity: Entity,
    ) -> Result<(), AdditionError> {
        self.is_space_available(center, footprint, facing)?;

        for voxel_pos in footprint.normalized(facing, center) {
            let voxel_data = VoxelObject {
                entity: structure_entity,
                object_kind: VoxelKind::Structure {
                    can_walk_on_roof,
                    can_walk_through,
                },
            };
            self.voxel_index.insert(voxel_pos, voxel_data);

            self.recompute_walkable_neighbors();
        }

        #[cfg(test)]
        self.validate();

        Ok(())
    }

    /// Removes any structure entity found at the provided `voxel_pos` from the voxel index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_structure(
        &mut self,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> Option<Entity> {
        let tentative_entry = self.voxel_index.get(&center)?;
        if !matches!(tentative_entry.object_kind, VoxelKind::Structure { .. }) {
            return None;
        }

        let entity = tentative_entry.entity;

        for voxel_pos in footprint.normalized(facing, center) {
            self.voxel_index.remove(&voxel_pos);
        }

        self.recompute_walkable_neighbors();

        #[cfg(test)]
        self.validate();

        Some(entity)
    }

    /// Adds the provided `litter_entity` to the voxel index at the provided `center`.
    ///
    /// If the voxel is not clear, the litter will be placed in the nearest empty voxel on the ground.
    #[inline]
    pub(crate) fn add_litter(
        &mut self,
        voxel_pos: VoxelPos,
        inventory_state: InventoryState,
        litter_entity: Entity,
        // FIXME: this should not return a result
    ) -> Result<(), AdditionError> {
        self.is_voxel_clear(voxel_pos)?;

        let voxel_data = VoxelObject {
            entity: litter_entity,
            object_kind: VoxelKind::Litter { inventory_state },
        };

        // FIXME: This overwrites the existing entry
        // Instead, litter should be placed in the nearest empty voxel on the ground
        self.voxel_index.insert(voxel_pos, voxel_data);
        self.recompute_walkable_neighbors();

        #[cfg(test)]
        self.validate();

        Ok(())
    }

    /// Removes any litter entity found at the provided `voxel_pos` from the voxel index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_litter(&mut self, voxel_pos: VoxelPos) -> Option<Entity> {
        let tentative_entry = self.voxel_index.get(&voxel_pos)?;
        if !matches!(tentative_entry.object_kind, VoxelKind::Litter { .. }) {
            return None;
        }

        let entity = tentative_entry.entity;
        self.voxel_index.remove(&voxel_pos);

        self.recompute_walkable_neighbors();

        #[cfg(test)]
        self.validate();

        Some(entity)
    }

    /// Moves the litter entity found at the provided `voxel_pos` to the provided `new_voxel_pos`.
    ///
    /// This operation is infallible: if the litter cannot be moved to the desired position,
    /// it will instead be droppped in the nearest empty voxel.
    #[inline]
    pub(crate) fn move_litter(&mut self, mut voxel_pos: Mut<VoxelPos>, new_voxel_pos: VoxelPos) {
        if *voxel_pos == new_voxel_pos {
            return;
        }

        let Some(litter_entity) = self.remove_litter(*voxel_pos) else { return; };
        self.add_litter(new_voxel_pos, InventoryState::Full, litter_entity)
            .unwrap();
        *voxel_pos = new_voxel_pos;
    }

    /// Gets the ghost structure [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_ghost_structure(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        let voxel_data = self.get_voxel(voxel_pos)?;
        match voxel_data.object_kind {
            VoxelKind::GhostStructure => Some(voxel_data.entity),
            _ => None,
        }
    }

    /// Adds the provided `ghost_structure_entity` to the voxel index at the provided `center`.
    #[inline]
    pub(crate) fn add_ghost_structure(
        &mut self,
        facing: Facing,
        center: VoxelPos,
        footprint: &Footprint,
        ghost_structure_entity: Entity,
    ) -> Result<(), AdditionError> {
        self.is_space_available(center, footprint, facing)?;

        for voxel_pos in footprint.normalized(facing, center) {
            let voxel_data = VoxelObject {
                entity: ghost_structure_entity,
                object_kind: VoxelKind::GhostStructure,
            };

            self.voxel_index.insert(voxel_pos, voxel_data);
        }

        // We do not need to update the passable neighbors, as ghost structures never block movement

        #[cfg(test)]
        self.validate();

        Ok(())
    }

    /// Removes any ghost structure entity found at the provided `voxel_pos` from the voxel index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_ghost_structure(
        &mut self,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> Option<Entity> {
        let tentative_entry = self.voxel_index.get(&center)?;
        if !matches!(tentative_entry.object_kind, VoxelKind::GhostStructure) {
            return None;
        }

        let entity = tentative_entry.entity;

        for voxel_pos in footprint.normalized(facing, center) {
            self.voxel_index.remove(&voxel_pos);
        }

        self.recompute_walkable_neighbors();

        #[cfg(test)]
        self.validate();

        Some(entity)
    }

    /// Returns an iterator over all of the hex positions that are ocean tiles.
    #[inline]
    #[must_use]
    pub(crate) fn ocean_tiles(&self) -> impl ExactSizeIterator<Item = Hex> + '_ {
        // Oceans ring the entire map currently
        Hex::ZERO.ring(self.radius + 1)
    }

    /// The set of tiles adjacent to `hex` that are on the map.
    #[inline]
    #[must_use]
    pub(crate) fn adjacent_hexes(&self, hex: Hex) -> [Option<Hex>; 6] {
        let mut adjacent_hexes = [None; 6];

        for (i, neighbor) in hex.ring(1).enumerate() {
            if self.is_valid(neighbor) {
                adjacent_hexes[i] = Some(neighbor);
            }
        }

        adjacent_hexes
    }

    /// The set of tiles that can be walked to by a basket crab from `voxel_pos`.
    ///
    /// The function signature is unfortunate, but this is meaningfully faster in a hot loop than returning a vec of tile positions.
    #[inline]
    #[must_use]
    pub(crate) fn walkable_neighbors(&self, voxel_pos: VoxelPos) -> &[Option<VoxelPos>; 6] {
        self.walkable_neighbors
            .get(&voxel_pos)
            // Avoiding panics here is much more robust and makes debugging easier
            .unwrap_or(&self.empty_neighbors)
    }

    /// Computes the set of tiles across the entire map that can be walked on by a basket crab.
    pub(crate) fn walkable_voxels(&self) -> HashSet<VoxelPos> {
        let mut walkable_voxels = HashSet::new();

        for (voxel_pos, voxel_data) in self.voxel_index.iter() {
            if voxel_data.object_kind.can_walk_on_roof() {
                let can_walk_through = match self.get_voxel(voxel_pos.above()) {
                    Some(voxel_data) => voxel_data.object_kind.can_walk_through(),
                    None => true,
                };

                if can_walk_through {
                    walkable_voxels.insert(voxel_pos.above());
                }
            }
        }

        walkable_voxels
    }

    /// Recomputes the set of passable neighbors for the provided `voxel_pos`.
    ///
    /// This will update the entire map at once.
    // PERF: only update the neighborhood of the provided `voxel_pos`
    fn recompute_walkable_neighbors(&mut self) {
        let walkable_voxels = self.walkable_voxels();
        self.walkable_neighbors.clear();

        for walkable_voxel in &walkable_voxels {
            let mut local_neighbors = [None; 6];

            for (i, &direction) in hexx::Direction::ALL_DIRECTIONS.iter().enumerate() {
                let neighbor_hex = walkable_voxel.hex.neighbor(direction);
                let neighbor_flat = VoxelPos {
                    hex: neighbor_hex,
                    height: walkable_voxel.height,
                };
                let neighbor_above = neighbor_flat.above();
                let neighbor_below = neighbor_flat.below();

                // Preferentially walk up, then level, then down
                // So far, this is an arbitrary priority system
                local_neighbors[i] = if walkable_voxels.contains(&neighbor_above) {
                    Some(neighbor_above)
                } else if walkable_voxels.contains(&neighbor_flat) {
                    Some(neighbor_flat)
                } else if walkable_voxels.contains(&neighbor_below) {
                    Some(neighbor_below)
                } else {
                    None
                }
            }

            self.walkable_neighbors
                .insert(*walkable_voxel, local_neighbors);
        }

        #[cfg(test)]
        self.validate();
    }
}

#[cfg(test)]
impl MapGeometry {
    /// Runs all of the validation checks on the map.
    fn validate(&self) {
        self.validate_heights();
        self.validate_entity_mapping();
        self.ensure_hex_keys_match();
        self.ensure_height_and_voxel_indexes_match();
        self.validate_walkable_voxels();
    }

    /// Asserts that all of the heights in the map are between `Height::ZERO` and `Height::MAX`.
    fn validate_heights(&self) {
        for voxel_pos in self.voxel_index.keys() {
            let height = voxel_pos.height();
            assert!(
                height >= Height::ZERO && height <= Height::MAX,
                "Height {} is out of range",
                height
            );
        }

        for &hex in self.all_hexes() {
            let height = self.get_height(hex).unwrap();
            assert!(
                height >= DiscreteHeight::ZERO && height <= DiscreteHeight::MAX,
                "Height {} is out of range",
                height
            );
        }
    }

    /// Asserts that the heights recorded for terrain in the voxel index match the height map.
    fn ensure_height_and_voxel_indexes_match(&self) {
        for (voxel_pos, voxel_object) in self.voxel_index.iter() {
            if !matches!(&voxel_object.object_kind, &VoxelKind::Terrain) {
                continue;
            }

            let voxel_height = voxel_pos.height;
            let hex = voxel_pos.hex;
            let stored_height = self.get_height(hex).unwrap();

            assert_eq!(
                voxel_height, stored_height,
                "Height mismatch at {}",
                voxel_pos
            );
        }
    }

    /// Asserts that the set of keys and values in the walkable neighbors index line up with the freshly computed set of walkable voxels.
    fn validate_walkable_voxels(&self) {
        let walkable_voxels = self.walkable_voxels().into_iter().collect::<HashSet<_>>();
        let walkable_neighbors_keys = self
            .walkable_neighbors
            .keys()
            .copied()
            .collect::<HashSet<_>>();

        let a_minus_b = walkable_voxels.difference(&walkable_neighbors_keys);
        let b_minus_a = walkable_neighbors_keys.difference(&walkable_voxels);

        assert!(
            walkable_voxels == walkable_neighbors_keys,
            "Walkable voxels and walkable neighbors keys do not match. Found {:?} in walkable voxels but not in walkable neighbors keys. Found {:?} in walkable neighbors keys but not in walkable voxels.",
            a_minus_b, b_minus_a
        );

        for neighbors in self.walkable_neighbors.values() {
            for maybe_neighbor in neighbors.iter().flatten() {
                assert!(walkable_voxels.contains(maybe_neighbor));
            }
        }
    }

    /// Asserts that the keys in the height index and the terrain index match.
    fn ensure_hex_keys_match(&self) {
        assert_eq!(
            self.height_index.keys().collect::<HashSet<_>>(),
            self.terrain_index.keys().collect::<HashSet<_>>(),
            "Height index keys do not match terrain index keys"
        );
    }

    /// Asserts that the entities recorded in the voxel index match the entities recorded in the terrain map.
    fn validate_entity_mapping(&self) {
        for (voxel_pos, voxel_object) in self.voxel_index.iter() {
            if !matches!(&voxel_object.object_kind, &VoxelKind::Terrain) {
                continue;
            }

            let voxel_entity = voxel_object.entity;
            let terrain_entity = self.get_terrain(voxel_pos.hex).unwrap();

            assert_eq!(
                voxel_entity, terrain_entity,
                "Entity mismatch at {}",
                voxel_pos
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::position::DiscreteHeight;

    use super::*;

    #[test]
    fn map_geometry_is_initialized_successfully() {
        let radius = 10;

        let mut world = World::new();
        let map_geometry = MapGeometry::new(&mut world, radius);
        let hexagon = hexagon(Hex::ZERO, radius);
        let n = hexagon.len();

        assert_eq!(map_geometry.radius, radius);
        // Valid neighbors is larger, as this information is needed for ocean tiles
        let n_walkable_neighbors = map_geometry.walkable_neighbors.iter().count();
        assert_eq!(n_walkable_neighbors, n);

        for hex in hexagon {
            let voxel_pos_zero = VoxelPos {
                hex,
                height: DiscreteHeight::ZERO,
            };
            let voxel_pos_one = VoxelPos {
                hex,
                height: DiscreteHeight::ONE,
            };

            assert!(
                map_geometry.is_valid(hex),
                "All hexes in the hexagon should be valid"
            );

            assert!(
                map_geometry.terrain_index.contains_key(&hex),
                "Terrain index should contain {:?}",
                hex
            );

            assert!(
                map_geometry.height_index.contains_key(&hex),
                "Height index should contain {:?}",
                hex
            );

            assert_eq!(
                map_geometry
                    .voxel_index
                    .get(&voxel_pos_zero)
                    .unwrap()
                    .object_kind,
                VoxelKind::Terrain
            );
            assert_eq!(
                map_geometry.is_voxel_clear(voxel_pos_zero),
                Err(AdditionError::AlreadyOccupied)
            );

            assert_eq!(map_geometry.voxel_index.get(&voxel_pos_one), None);
            assert_eq!(map_geometry.is_voxel_clear(voxel_pos_one), Ok(()));

            assert!(
                map_geometry.walkable_neighbors.contains_key(&voxel_pos_one),
                "Walkable neighbors should contain {}",
                voxel_pos_one
            );

            assert_eq!(
                map_geometry.get_height(hex),
                Ok(DiscreteHeight::ZERO),
                "All hexes should be at height 0"
            );
        }

        assert_eq!(
            map_geometry.walkable_voxels().len(),
            n,
            "All hexes should correspond to one walkable position"
        );

        for (voxel_pos, valid_neighbors) in &map_geometry.walkable_neighbors {
            assert!(valid_neighbors.len() <= 6, "{}", voxel_pos);
            for maybe_neighbor in valid_neighbors {
                if let Some(neighbor) = maybe_neighbor {
                    assert!(map_geometry.is_valid(neighbor.hex), "{}", neighbor);

                    assert!(
                        map_geometry.walkable_neighbors.contains_key(neighbor),
                        "{}",
                        neighbor
                    );
                }
            }
        }

        map_geometry.validate();
    }

    #[test]
    fn walkable_voxels_respond_to_changes_correctly() {
        let mut map_geometry = MapGeometry::new(&mut World::new(), 0);
        let can_walk_at_height_one = HashSet::from_iter([VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight(1),
        }]);
        let can_walk_at_height_two = HashSet::from_iter([VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight(2),
        }]);
        let cannot_walk = HashSet::new();

        let center = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight(1),
        };
        let footprint = Footprint::default();
        let facing = Facing::default();
        let entity = Entity::from_bits(42);

        assert_eq!(map_geometry.walkable_voxels(), can_walk_at_height_one);
        map_geometry.is_voxel_clear(center).unwrap();
        map_geometry
            .is_space_available(center, &footprint, facing)
            .unwrap();

        // Ordinary structure
        map_geometry
            .add_structure(center, facing, &footprint, false, false, entity)
            .unwrap();

        assert_eq!(map_geometry.walkable_voxels(), cannot_walk);
        map_geometry
            .remove_structure(center, &footprint, facing)
            .unwrap();

        // Ghost structure
        map_geometry
            .add_ghost_structure(facing, center, &footprint, entity)
            .unwrap();

        assert_eq!(map_geometry.walkable_voxels(), can_walk_at_height_one);
        map_geometry.remove_ghost_structure(center, &footprint, facing);

        // Passable structure
        map_geometry
            .add_structure(center, facing, &footprint, false, true, entity)
            .unwrap();

        assert_eq!(map_geometry.walkable_voxels(), can_walk_at_height_one);
        map_geometry
            .remove_structure(center, &footprint, facing)
            .unwrap();

        // Structure with roof
        map_geometry
            .add_structure(center, facing, &footprint, true, false, entity)
            .unwrap();

        assert_eq!(map_geometry.walkable_voxels(), can_walk_at_height_two);

        map_geometry.remove_structure(center, &footprint, facing);

        // Raising the terrain
        map_geometry.update_height(Hex::ZERO, DiscreteHeight::ONE);

        assert_eq!(map_geometry.walkable_voxels(), can_walk_at_height_two);
    }

    #[test]
    fn adding_ghost_structures_does_not_change_walkable_neighbors() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 1);
        let voxel_pos = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight::ONE,
        };
        let facing = Facing::default();
        let footprint = Footprint::default();

        let initial_walkable_neighbors = map_geometry.walkable_neighbors.clone();

        map_geometry
            .add_ghost_structure(facing, voxel_pos, &footprint, Entity::from_bits(42))
            .unwrap();
        let initial_walkable_voxels = map_geometry.walkable_voxels();

        assert_eq!(
            map_geometry.walkable_neighbors, initial_walkable_neighbors,
            "Adding a ghost structure should not change walkable neighbors"
        );

        assert_eq!(
            initial_walkable_voxels,
            map_geometry.walkable_voxels(),
            "Adding a ghost structure should not change walkable voxels"
        );

        map_geometry.remove_ghost_structure(voxel_pos, &footprint, facing);

        assert_eq!(
            map_geometry.walkable_neighbors, initial_walkable_neighbors,
            "Removing a ghost structure should not change walkable neighbors"
        );
    }

    #[test]
    fn adding_passable_structures_does_not_change_walkable_neighbors() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 1);
        let voxel_pos = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight::ONE,
        };
        let facing = Facing::default();
        let footprint = Footprint::default();

        let initial_walkable_neighbors = map_geometry.walkable_neighbors.clone();

        map_geometry
            .add_structure(
                voxel_pos,
                facing,
                &footprint,
                false,
                true,
                Entity::from_bits(42),
            )
            .unwrap();
        let initial_walkable_voxels = map_geometry.walkable_voxels();

        assert_eq!(
            map_geometry.walkable_neighbors, initial_walkable_neighbors,
            "Adding a passable structure should not change walkable neighbors"
        );

        assert_eq!(
            initial_walkable_voxels,
            map_geometry.walkable_voxels(),
            "Adding a passable structure should not change walkable voxels"
        );

        map_geometry.remove_structure(voxel_pos, &footprint, facing);

        assert_eq!(
            map_geometry.walkable_neighbors, initial_walkable_neighbors,
            "Removing a ghost structure should not change walkable neighbors"
        );
    }

    #[test]
    fn can_add_and_remove_structures() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 0);
        let voxel_pos = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight::ONE,
        };
        let facing = Facing::default();
        let footprint = Footprint::default();

        map_geometry
            .add_structure(
                voxel_pos,
                facing,
                &footprint,
                false,
                false,
                Entity::from_bits(42),
            )
            .unwrap();

        assert_eq!(
            map_geometry.get_structure(voxel_pos),
            Some(Entity::from_bits(42))
        );

        map_geometry.remove_structure(voxel_pos, &footprint, facing);

        assert_eq!(map_geometry.get_structure(voxel_pos), None);
    }

    #[test]
    fn can_add_and_remove_ghost_structures() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 0);
        let voxel_pos = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight::ONE,
        };
        let facing = Facing::default();
        let footprint = Footprint::default();

        map_geometry
            .add_ghost_structure(facing, voxel_pos, &footprint, Entity::from_bits(42))
            .unwrap();

        assert_eq!(
            map_geometry.get_ghost_structure(voxel_pos),
            Some(Entity::from_bits(42))
        );

        map_geometry.remove_ghost_structure(voxel_pos, &footprint, facing);

        assert_eq!(map_geometry.get_ghost_structure(voxel_pos), None);
    }

    #[test]
    fn can_change_height_of_terrain() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 0);
        assert_eq!(
            map_geometry.get_height(Hex::ZERO).unwrap(),
            DiscreteHeight::ZERO
        );

        map_geometry.update_height(Hex::ZERO, DiscreteHeight::ONE);
        assert_eq!(
            map_geometry.get_height(Hex::ZERO).unwrap(),
            DiscreteHeight::ONE
        );
    }

    // TODO: add tests for litter

    #[test]
    fn adding_multi_tile_structure_adds_to_index() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 0);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let center = VoxelPos {
            hex: Hex::ZERO,
            height: DiscreteHeight::ONE,
        };
        let can_walk_on_roof = false;
        let can_walk_through = false;

        map_geometry
            .add_structure(
                center,
                facing,
                &footprint,
                can_walk_on_roof,
                can_walk_through,
                structure_entity,
            )
            .unwrap();

        // Check that the structure index was updated correctly
        for voxel_pos in footprint.normalized(facing, center) {
            assert_eq!(
                Some(structure_entity),
                map_geometry.get_structure(voxel_pos)
            );
        }
    }

    #[test]
    fn removing_multi_tile_structure_clears_indexes() {
        let mut world = World::new();
        let mut map_geometry = MapGeometry::new(&mut world, 0);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let hex = Hex { x: 3, y: -2 };
        let center = VoxelPos {
            hex,
            height: DiscreteHeight::ZERO,
        };
        let can_walk_on_roof = false;
        let can_walk_through = false;

        map_geometry
            .add_structure(
                center,
                facing,
                &footprint,
                can_walk_on_roof,
                can_walk_through,
                structure_entity,
            )
            .unwrap();
        map_geometry.remove_structure(center, &footprint, facing);

        // Check that the structure index was updated correctly
        for voxel_pos in footprint.normalized(facing, center) {
            dbg!(voxel_pos);
            assert_eq!(None, map_geometry.get_structure(voxel_pos));
        }
    }
}
