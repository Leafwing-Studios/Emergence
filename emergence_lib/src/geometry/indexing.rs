//! Tracks the location of key entities on the map, and caches information about the map for faster access.

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use hexx::{shapes::hexagon, Hex, HexLayout};

use crate::{
    items::inventory::InventoryState, structures::Footprint, units::actions::DeliveryMode,
};

use super::{Facing, Height, VoxelObject, VoxelPos};

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
    terrain_index: HashMap<Hex, Entity>,
    /// Tracks which objects are stored in each voxel.
    voxel_index: HashMap<VoxelPos, VoxelObject>,
    /// Which [`Id<Structure>`](crate::asset_management::manifest::Id) entity is stored at each tile position
    structure_index: HashMap<VoxelPos, Entity>,
    /// Which [`Ghost`](crate::construction::ghosts::Ghost) structure entity is stored at each tile position
    ghost_structure_index: HashMap<VoxelPos, Entity>,
    /// Which [`Ghost`](crate::construction::ghosts::Ghost) terrain entity is stored at each tile position
    ghost_terrain_index: HashMap<VoxelPos, Entity>,
    /// The set of tiles that cannot be traversed by units due to structures.
    impassable_structure_tiles: HashSet<VoxelPos>,
    /// The set of tiles that cannot be traversed by units due to litter.
    impassable_litter_tiles: HashSet<VoxelPos>,
    /// The height of the terrain at each tile position.
    height_index: HashMap<Hex, Height>,
    /// The list of all valid neighbors for each tile position.
    valid_neighbors: HashMap<VoxelPos, [Option<VoxelPos>; 6]>,
    /// The list of all passable neighbors for each tile position.
    passable_neighbors: HashMap<VoxelPos, [Option<VoxelPos>; 6]>,
    /// The list of all reachable neighbors for each tile position.
    reachable_neighbors: HashMap<VoxelPos, [Option<VoxelPos>; 6]>,
}

/// A [`MapGeometry`] index was missing an entry.
#[derive(Debug, PartialEq)]
pub struct IndexError {
    /// The hex where the data was missing.
    pub hex: Hex,
}

impl MapGeometry {
    /// Creates a new [`MapGeometry`] of the provided raidus.
    ///
    /// All indexes will be empty.
    pub fn new(radius: u32) -> Self {
        let hexes: Vec<Hex> = hexagon(Hex::ZERO, radius).collect();
        let tiles: Vec<VoxelPos> = hexes
            .iter()
            .map(|hex| VoxelPos::new(*hex, Height::MIN))
            .collect();

        // We can start with the minimum height everywhere as no entities need to be spawned.
        let height_index = tiles
            .iter()
            .map(|voxel_pos| (voxel_pos.hex, Height::MIN))
            .collect();

        let reachable_neighbors: HashMap<VoxelPos, [Option<VoxelPos>; 6]> = hexes
            .iter()
            .map(|hex| {
                let voxel_pos = VoxelPos::new(*hex, Height::MIN);
                let mut neighbors = [None; 6];

                for (i, neighboring_hex) in hex.all_neighbors().into_iter().enumerate() {
                    if Hex::ZERO.distance_to(neighboring_hex) <= radius as i32 {
                        neighbors[i] = Some(VoxelPos::new(neighboring_hex, Height::MIN))
                    }
                }

                (voxel_pos, neighbors)
            })
            .collect();

        let passable_neighbors = reachable_neighbors.clone();
        let mut valid_neighbors = reachable_neighbors.clone();

        // Define valid neighbors for ocean tiles
        for hex in Hex::ZERO.ring(radius + 1) {
            let voxel_pos = VoxelPos::new(hex, Height::MIN);
            let mut neighbors = [None; 6];
            for (i, neighboring_hex) in hex.all_neighbors().into_iter().enumerate() {
                if Hex::ZERO.distance_to(neighboring_hex) <= radius as i32 {
                    neighbors[i] = Some(VoxelPos::new(neighboring_hex, Height::MIN))
                }
            }

            valid_neighbors.insert(voxel_pos, neighbors);
        }

        MapGeometry {
            layout: HexLayout::default(),
            radius,
            terrain_index: HashMap::default(),
            voxel_index: HashMap::default(),
            structure_index: HashMap::default(),
            ghost_structure_index: HashMap::default(),
            ghost_terrain_index: HashMap::default(),
            impassable_structure_tiles: HashSet::default(),
            impassable_litter_tiles: HashSet::default(),
            valid_neighbors,
            passable_neighbors,
            reachable_neighbors,
            height_index,
        }
    }

    /// Returns the list of all valid [`Hex`] positions on the map.
    #[inline]
    #[must_use]
    pub fn all_hexes(&self) -> impl Iterator<Item = &Hex> {
        self.terrain_index.keys()
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
    pub(crate) fn get_voxel_object(&self, voxel_pos: VoxelPos) -> Option<&VoxelObject> {
        self.voxel_index.get(&voxel_pos)
    }

    /// Can we walk through the provide `voxel_pos`?
    #[inline]
    #[must_use]
    pub(crate) fn can_walk_through(&self, voxel_pos: VoxelPos) -> bool {
        self.get_voxel_object(voxel_pos)
            .map(|voxel_object| voxel_object.object_kind.can_walk_through())
            // If there's nothing there, it's air and we can walk through it
            .unwrap_or(true)
    }

    /// Can we walk on top of the provided `voxel_pos`?
    #[inline]
    #[must_use]
    pub(crate) fn can_walk_on_top_of(&self, voxel_pos: VoxelPos) -> bool {
        self.get_voxel_object(voxel_pos)
            .map(|voxel_object| voxel_object.object_kind.can_walk_on_top_of())
            // If there's nothing there, it's air and we can't walk on top of it
            .unwrap_or(false)
    }

    /// Can we walk at the provided `voxel_pos`?
    #[inline]
    #[must_use]
    pub(crate) fn can_walk_at(&self, voxel_pos: VoxelPos) -> bool {
        self.can_walk_through(voxel_pos) && self.can_walk_on_top_of(voxel_pos.below())
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

        if self.impassable_structure_tiles.contains(&ending_pos) {
            return false;
        }

        if self.impassable_litter_tiles.contains(&ending_pos) {
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
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        footprint
            .normalized(facing, center)
            .iter()
            .all(|voxel_pos| self.get_structure(*voxel_pos).is_none())
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
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) -> bool {
        self.is_footprint_valid(center, footprint, facing)
            && self.is_terrain_flat(center, footprint, facing)
            && self.is_space_available(center, footprint, facing)
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

    /// Updates the height of the tile at `voxel_pos`
    #[inline]
    pub fn update_height(&mut self, hex: Hex, height: Height) {
        assert!(
            self.is_valid(hex),
            "Invalid tile position: {:?} with a radius of {:?}",
            hex,
            self.radius
        );
        assert!(height >= Height(0.));
        self.height_index.insert(hex, height);

        let voxel_pos = VoxelPos::new(hex, height);

        // FIXME: this should update the voxel index, which should *then* trigger a recompute of the neighbors
        self.recompute_passable_neighbors(voxel_pos);
        self.recompute_reachable_neighbors(voxel_pos);
    }

    /// Returns the height of the tile at `voxel_pos`, if available.
    ///
    /// This should always be [`Ok`] for all valid tiles.
    pub(crate) fn get_height(&self, hex: Hex) -> Result<Height, IndexError> {
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

    /// Returns the absolute difference in height between the tile at `starting_pos` and the tile at `ending_pos`.
    #[inline]
    pub(crate) fn height_difference(
        &self,
        starting_pos: VoxelPos,
        ending_pos: VoxelPos,
    ) -> Result<Height, IndexError> {
        // FIXME: this doesn't need self at all!
        let starting_height = starting_pos.height();
        let ending_height = ending_pos.height();
        Ok(starting_height.abs_diff(ending_height))
    }

    /// Flattens the terrain in the `footprint` around `voxel_pos` to the height at that location.
    ///
    /// This footprint is rotated by the supplied `facing`.
    pub(crate) fn flatten_height(
        &mut self,
        height_query: &mut Query<&mut Height>,
        center: VoxelPos,
        footprint: &Footprint,
        facing: Facing,
    ) {
        let Ok(target_height) = self.get_height(center.hex) else { return };
        for voxel_pos in footprint.normalized(facing, center) {
            if let Some(entity) = self.get_terrain(voxel_pos.hex) {
                if let Ok(mut height) = height_query.get_mut(entity) {
                    *height = target_height;
                    self.update_height(voxel_pos.hex, target_height);
                }
            }
        }
    }

    /// Gets the [`Entity`] at the provided `voxel_pos` that might have or want an item.
    ///
    /// If the `delivery_mode` is [`DeliveryMode::PickUp`], looks for litter, ghost terrain, or structures.
    /// If the `delivery_mode` is [`DeliveryMode::DropOff`], looks for ghost structures, ghost terrain or structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_candidates(
        &self,
        voxel_pos: VoxelPos,
        delivery_mode: DeliveryMode,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();

        match delivery_mode {
            DeliveryMode::DropOff => {
                if let Some(&structure_entity) = self.structure_index.get(&voxel_pos) {
                    entities.push(structure_entity)
                }

                if let Some(&ghost_terrain_entity) = self.ghost_terrain_index.get(&voxel_pos) {
                    entities.push(ghost_terrain_entity)
                }

                if let Some(&ghost_structure_entity) = self.ghost_structure_index.get(&voxel_pos) {
                    entities.push(ghost_structure_entity)
                }
            }
            DeliveryMode::PickUp => {
                if let Some(&structure_entity) = self.structure_index.get(&voxel_pos) {
                    entities.push(structure_entity)
                }

                if let Some(&ghost_terrain_entity) = self.ghost_terrain_index.get(&voxel_pos) {
                    entities.push(ghost_terrain_entity)
                }

                if let Some(&litter_entity) = self.terrain_index.get(&voxel_pos.hex) {
                    // FIXME: this should not be stored on terrain
                    entities.push(litter_entity)
                }
            }
        }

        entities
    }

    /// Gets entities that units might work at, at the provided `voxel_pos`.
    ///
    /// Prioritizes ghosts over structures if both are present to allow for replacing structures.
    #[inline]
    #[must_use]
    pub(crate) fn get_workplaces(&self, voxel_pos: VoxelPos) -> Vec<Entity> {
        let mut entities = Vec::new();

        if let Some(&ghost_structure_entity) = self.ghost_structure_index.get(&voxel_pos) {
            entities.push(ghost_structure_entity)
        }

        if let Some(&structure_entity) = self.structure_index.get(&voxel_pos) {
            entities.push(structure_entity)
        }

        entities
    }

    /// Gets the terrain [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_terrain(&self, hex: Hex) -> Option<Entity> {
        self.terrain_index.get(&hex).copied()
    }

    /// Adds the provided `terrain_entity` to the terrain index at the provided `voxel_pos`.
    ///
    /// This also updates the height map.
    #[inline]
    pub fn add_terrain(&mut self, voxel_pos: VoxelPos, terrain_entity: Entity) {
        let hex = voxel_pos.hex;
        let height = voxel_pos.height();

        self.terrain_index.insert(hex, terrain_entity);
        self.height_index.insert(hex, height);
    }

    /// Gets the structure [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_structure(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        self.structure_index.get(&voxel_pos).copied()
    }

    /// Adds the provided `structure_entity` to the structure index at the provided `center`.
    #[inline]
    pub(crate) fn add_structure(
        &mut self,
        facing: Facing,
        center: VoxelPos,
        footprint: &Footprint,
        passable: bool,
        structure_entity: Entity,
    ) {
        for voxel_pos in footprint.normalized(facing, center) {
            self.structure_index.insert(voxel_pos, structure_entity);
            if !passable {
                self.impassable_structure_tiles.insert(voxel_pos);
            }

            self.recompute_passable_neighbors(voxel_pos);
            self.recompute_reachable_neighbors(voxel_pos);
        }
    }

    /// Removes any structure entity found at the provided `voxel_pos` from the structure index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_structure(
        &mut self,
        facing: Facing,
        center: VoxelPos,
        footprint: &Footprint,
    ) -> Option<Entity> {
        let mut removed = None;

        for voxel_pos in footprint.normalized(facing, center) {
            removed = self.structure_index.remove(&voxel_pos);
            // We can do this even for passable structures, since we have a guarantee that only one structure can be at a tile
            // If that occurs, this fails silently, but that's intended behavior
            self.impassable_structure_tiles.remove(&voxel_pos);

            self.recompute_passable_neighbors(voxel_pos);
            self.recompute_reachable_neighbors(voxel_pos);
        }

        removed
    }

    /// Gets the ghost structure [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_ghost_structure(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        self.ghost_structure_index.get(&voxel_pos).copied()
    }

    /// Adds the provided `ghost_structure_entity` to the ghost structure index at the provided `center`.
    #[inline]
    pub(crate) fn add_ghost_structure(
        &mut self,
        facing: Facing,
        center: VoxelPos,
        footprint: &Footprint,
        ghost_structure_entity: Entity,
    ) {
        for voxel_pos in footprint.normalized(facing, center) {
            self.ghost_structure_index
                .insert(voxel_pos, ghost_structure_entity);
        }
    }

    /// Removes any ghost structure entity found at the provided `voxel_pos` from the ghost structure index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_ghost_structure(&mut self, voxel_pos: VoxelPos) -> Option<Entity> {
        let removed = self.ghost_structure_index.remove(&voxel_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.ghost_structure_index
                .retain(|_k, v| *v != removed_entity);
        };

        removed
    }

    /// Adds the provided `ghost_terrain_entity` to the ghost terrain index at the provided `voxel_pos`.
    #[inline]
    pub(crate) fn add_ghost_terrain(&mut self, ghost_terrain_entity: Entity, voxel_pos: VoxelPos) {
        self.ghost_terrain_index
            .insert(voxel_pos, ghost_terrain_entity);
    }

    /// Removes any ghost terrain entity found at the provided `voxel_pos` from the ghost terrain index.
    ///
    /// Returns the removed entity, if any.
    #[inline]
    pub(crate) fn remove_ghost_terrain(&mut self, voxel_pos: VoxelPos) -> Option<Entity> {
        let removed = self.ghost_terrain_index.remove(&voxel_pos);

        // Iterate through all of the entries, removing any other entries that point to the same entity
        // PERF: this could be faster, but would require a different data structure.
        if let Some(removed_entity) = removed {
            self.ghost_terrain_index
                .retain(|_k, v| *v != removed_entity);
        };

        removed
    }

    /// Gets the ghost terrain [`Entity`] at the provided `voxel_pos`, if any.
    #[inline]
    #[must_use]
    pub(crate) fn get_ghost_terrain(&self, voxel_pos: VoxelPos) -> Option<Entity> {
        self.ghost_terrain_index.get(&voxel_pos).copied()
    }

    /// Updates the passability of the provided `voxel_pos` based on the state of the litter at that location.
    pub(crate) fn update_litter_state(
        &mut self,
        voxel_pos: VoxelPos,
        litter_state: InventoryState,
    ) {
        let current_litter_state = self.impassable_litter_tiles.contains(&voxel_pos);

        match current_litter_state {
            true => {
                if litter_state != InventoryState::Full {
                    self.impassable_litter_tiles.remove(&voxel_pos);
                    self.recompute_passable_neighbors(voxel_pos);
                    self.recompute_reachable_neighbors(voxel_pos);
                }
            }
            false => {
                if litter_state == InventoryState::Full {
                    self.impassable_litter_tiles.insert(voxel_pos);
                    self.recompute_passable_neighbors(voxel_pos);
                    self.recompute_reachable_neighbors(voxel_pos);
                }
            }
        }
    }

    /// Returns an iterator over all of the hex positions that are ocean tiles.
    #[inline]
    #[must_use]
    pub(crate) fn ocean_tiles(&self) -> impl ExactSizeIterator<Item = Hex> + '_ {
        // Oceans ring the entire map currently
        Hex::ZERO.ring(self.radius + 1)
    }

    /// The set of adjacent tiles that are on the map.
    ///
    /// # Panics
    ///
    /// The provided `voxel_pos` must be a valid tile position.
    // FIXME: this should return `Hex`
    #[inline]
    #[must_use]
    pub(crate) fn valid_neighbors(&self, voxel_pos: VoxelPos) -> &[Option<VoxelPos>; 6] {
        self.valid_neighbors
            .get(&voxel_pos)
            .unwrap_or_else(|| panic!("Tile position {voxel_pos:?} is not a valid tile position"))
    }

    /// The set of tiles that can be walked to by a basket crab from `voxel_pos`.
    ///
    /// The function signature is unfortunate, but this is meaningfully faster in a hot loop than returning a vec of tile positions.
    ///
    /// # Panics
    ///
    /// The provided `voxel_pos` must be a valid tile position.
    #[inline]
    #[must_use]
    pub(crate) fn passable_neighbors(&self, voxel_pos: VoxelPos) -> &[Option<VoxelPos>; 6] {
        self.passable_neighbors
            .get(&voxel_pos)
            .unwrap_or_else(|| panic!("Tile position {voxel_pos:?} is not a valid tile position"))
    }

    /// The set of tiles that can be reached by a basket crab from `voxel_pos`.
    ///
    /// # Panics
    ///
    /// The provided `voxel_pos` must be a valid tile position.
    #[inline]
    #[must_use]
    pub(crate) fn reachable_neighbors(&self, voxel_pos: VoxelPos) -> &[Option<VoxelPos>; 6] {
        self.reachable_neighbors
            .get(&voxel_pos)
            .unwrap_or_else(|| panic!("Tile position {voxel_pos:?} is not a valid tile position"))
    }

    /// Recomputes the set of passable neighbors for the provided `voxel_pos`.
    ///
    /// This will update the provided tile and all of its neighbors.
    fn recompute_passable_neighbors(&mut self, voxel_pos: VoxelPos) {
        let neighbors = *self.valid_neighbors(voxel_pos);
        let mut passable_neighbors: [Option<VoxelPos>; 6] = [None; 6];

        for (i, maybe_neighbor) in neighbors.iter().enumerate() {
            let &Some(neighbor) = maybe_neighbor else { continue };

            let can_pass_from_tile_to_neighbor = self.compute_passability(voxel_pos, neighbor);
            let can_pass_from_neighbor_to_tile = self.compute_passability(neighbor, voxel_pos);

            match can_pass_from_tile_to_neighbor {
                true => {
                    passable_neighbors[i] = Some(neighbor);
                }
                // This edge was already initialized as None
                false => (),
            }

            let valid_neighbors_of_neighbor = self.valid_neighbors(neighbor);
            // PERF: we could compute this faster by relying on
            let index_of_self_in_neighbor = valid_neighbors_of_neighbor
                .iter()
                .position(|&maybe_self| maybe_self == Some(voxel_pos))
                .unwrap();
            let neigbors_of_neighbor = self.passable_neighbors.get_mut(&neighbor).unwrap();

            match can_pass_from_neighbor_to_tile {
                true => {
                    neigbors_of_neighbor[index_of_self_in_neighbor] = Some(voxel_pos);
                }
                false => {
                    neigbors_of_neighbor[index_of_self_in_neighbor] = None;
                }
            }
        }

        self.passable_neighbors
            .insert(voxel_pos, passable_neighbors);
    }

    /// Recomputes the set of reachable neighbors for the provided `voxel_pos`.
    ///
    /// This will update the provided tile and all of its neighbors.
    fn recompute_reachable_neighbors(&mut self, voxel_pos: VoxelPos) {
        let neighbors = *self.valid_neighbors(voxel_pos);
        let mut reachable_neighbors: [Option<VoxelPos>; 6] = [None; 6];

        for (i, maybe_neighbor) in neighbors.iter().enumerate() {
            let &Some(neighbor) = maybe_neighbor else { continue };

            let can_reach_from_tile_to_neighbor = self.compute_reachability(voxel_pos, neighbor);
            let can_reach_from_neighbor_to_tile = self.compute_reachability(neighbor, voxel_pos);

            match can_reach_from_tile_to_neighbor {
                true => {
                    reachable_neighbors[i] = Some(neighbor);
                }
                // This edge was already initialized as None
                false => (),
            }

            let valid_neighbors_of_neighbor = self.valid_neighbors(neighbor);
            // PERF: we could compute this faster by relying on
            let index_of_self_in_neighbor = valid_neighbors_of_neighbor
                .iter()
                .position(|&maybe_self| maybe_self == Some(voxel_pos))
                .unwrap();
            let neigbors_of_neighbor = self.reachable_neighbors.get_mut(&neighbor).unwrap();

            match can_reach_from_neighbor_to_tile {
                true => {
                    neigbors_of_neighbor[index_of_self_in_neighbor] = Some(voxel_pos);
                }
                false => {
                    neigbors_of_neighbor[index_of_self_in_neighbor] = None;
                }
            }
        }

        self.reachable_neighbors
            .insert(voxel_pos, reachable_neighbors);
    }

    /// Can the tile at `ending_pos` be moved to from the tile at `starting_pos`?
    fn compute_passability(&self, starting_pos: VoxelPos, ending_pos: VoxelPos) -> bool {
        if !self.is_valid(ending_pos.hex) {
            return false;
        }

        if self.impassable_structure_tiles.contains(&ending_pos) {
            return false;
        }

        if self.impassable_litter_tiles.contains(&ending_pos) {
            return false;
        }

        if let Ok(height_difference) = self.height_difference(starting_pos, ending_pos) {
            height_difference <= Height::MAX_STEP
        } else {
            false
        }
    }

    /// Can the tile at `ending_pos` be reached from the tile at `starting_pos`?
    // FIXME: this should be removed
    fn compute_reachability(&self, starting_pos: VoxelPos, ending_pos: VoxelPos) -> bool {
        if !self.is_valid(ending_pos.hex) {
            return false;
        }

        // TODO: does not take into account height of structures
        if let Ok(height_difference) = self.height_difference(starting_pos, ending_pos) {
            height_difference <= Height::MAX_STEP
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_geometry_is_initialized_successfully() {
        let radius = 10;

        let map_geometry = MapGeometry::new(radius);
        let hexagon = hexagon(Hex::ZERO, radius);
        let n = hexagon.len();

        assert_eq!(map_geometry.radius, radius);
        // Valid neighbors is larger, as this information is needed for ocean tiles
        let n_passable_neighbors = map_geometry.passable_neighbors.iter().count();
        assert_eq!(n_passable_neighbors, n);
        let n_reachable_neighbors = map_geometry.reachable_neighbors.iter().count();
        assert_eq!(n_reachable_neighbors, n);

        for hex in hexagon {
            let voxel_pos = VoxelPos::new(hex, Height::MIN);
            assert!(
                map_geometry.valid_neighbors.contains_key(&voxel_pos),
                "{}",
                voxel_pos
            );
            assert!(
                map_geometry.passable_neighbors.contains_key(&voxel_pos),
                "{}",
                voxel_pos
            );
            assert!(
                map_geometry.reachable_neighbors.contains_key(&voxel_pos),
                "{}",
                voxel_pos
            );
        }

        for (voxel_pos, valid_neighbors) in &map_geometry.valid_neighbors {
            assert!(valid_neighbors.len() <= 6, "{}", voxel_pos);
            for maybe_neighbor in valid_neighbors {
                if let Some(neighbor) = maybe_neighbor {
                    assert!(map_geometry.is_valid(neighbor.hex), "{}", neighbor);

                    assert!(
                        map_geometry.valid_neighbors.contains_key(neighbor),
                        "{}",
                        neighbor
                    );
                }
            }
        }

        // All of the neighbors should be the same for a newly initialized map
        assert_eq!(
            map_geometry.passable_neighbors,
            map_geometry.reachable_neighbors
        );
    }

    #[test]
    fn adding_multi_tile_structure_adds_to_index() {
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let center = VoxelPos::new(Hex::ZERO, Height::ZERO);
        let passable = false;

        map_geometry.add_structure(facing, center, &footprint, passable, structure_entity);

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
        let mut map_geometry = MapGeometry::new(10);

        let footprint = Footprint::hexagon(1);
        let structure_entity = Entity::from_bits(42);
        let facing = Facing::default();
        let hex = Hex { x: 3, y: -2 };
        let center = VoxelPos::new(hex, Height::ZERO);
        let passable = false;

        map_geometry.add_structure(facing, center, &footprint, passable, structure_entity);
        map_geometry.remove_structure(facing, center, &footprint);

        // Check that the structure index was updated correctly
        for voxel_pos in footprint.normalized(facing, center) {
            dbg!(voxel_pos);
            assert_eq!(None, map_geometry.get_structure(voxel_pos));
        }
    }
}
