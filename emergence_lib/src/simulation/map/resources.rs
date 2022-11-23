//! Code for managing data that is deeply tied to the map

use crate::simulation::map::neighbors::HexNeighbors;
use crate::simulation::map::MapPositions;
use bevy::prelude::Resource;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Spatial data for use with the [`MapResource`] struct.
// We cannot derive Clone on this, because:https://stackoverflow.com/questions/39415052/deriving-a-trait-results-in-unexpected-compiler-error-but-the-manual-implementa
#[derive(Debug)]
pub struct MapData<T> {
    pub(crate) inner: Arc<RwLock<T>>,
}

impl<T> Clone for MapData<T> {
    fn clone(&self) -> Self {
        MapData {
            inner: self.inner.clone(),
        }
    }
}

impl<T> MapData<T> {
    /// Create from data
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Immutably borrow the inner data (read access)
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.as_ref().read().unwrap()
    }

    /// Get mutable access to the data
    ///
    /// Will panic if write-access cannot be obtained
    /// TODO: flesh out under what conditions the panic will occur.
    pub fn get_mut(&mut self) -> RwLockWriteGuard<'_, T> {
        self.inner.write().unwrap()
    }
}

/// A helper for managing game resources that are naturally tied to a fixed specific position on
/// the map
///
/// It can give you [`MapData<T>`](MapData) at a given tile position, or it can give you
/// [`HexNeighbors<MapData<T>>`](HexNeighbors) for the given position.
///
/// Internally, [`MapData`] is stored in a [`HashMap`] for each position, in the `storage` field,
/// and this same data is then referenced to by the `neighbors` field.
#[derive(Resource)]
pub struct MapResource<T> {
    pub(crate) storage: HashMap<TilePos, MapData<T>>,
    pub(crate) neighbors: HashMap<TilePos, HexNeighbors<MapData<T>>>,
}

impl<T> MapResource<T>
where
    T: Default,
{
    /// Create new from an underlying `MapPostions` template
    ///
    /// This allocates capacity and initializes neighbors based on the template provided.
    ///
    /// This requires that that there is a `Default` impl for the underlying data type
    pub fn default_from_template(template: &MapPositions) -> MapResource<T> {
        let storage = MapResource::generate_storage(
            template,
            template
                .iter_positions()
                .map(|position| (*position, T::default())),
        );

        let neighbors = MapResource::generate_neighbors(&storage, template);

        MapResource { storage, neighbors }
    }
}

impl<T> MapResource<T> {
    /// Generate the storage [`HashMap`]
    pub fn generate_storage(
        template: &MapPositions,
        data: impl Iterator<Item = (TilePos, T)>,
    ) -> HashMap<TilePos, MapData<T>> {
        let mut storage = HashMap::with_capacity(template.n_positions());
        storage.extend(data.map(|(tile_pos, t)| (tile_pos, MapData::new(t))));
        storage
    }

    /// Generate neighbors for the given storage
    /// TODO: improve doc string
    pub fn generate_neighbors(
        storage: &HashMap<TilePos, MapData<T>>,
        template: &MapPositions,
    ) -> HashMap<TilePos, HexNeighbors<MapData<T>>> {
        let mut neighbors = HashMap::with_capacity(template.n_positions());
        neighbors.extend(template.iter_positions().filter_map(|position| {
            let tile_neighbors = template.get_neighbors(position)?;
            let neighbors = tile_neighbors.and_then_ref(|position| {
                let map_data = storage.get(position)?;
                let map_data_clone: MapData<T> = map_data.clone();
                Some(map_data_clone)
            });
            Some((*position, neighbors))
        }));
        neighbors
    }

    /// Create new from an underlying [`MapPostions`] template.
    ///
    /// This allocates capacity and initializes neighbors based on the template provided.
    ///
    /// If your underlying data implements [`Default`], you could use
    /// [`default_from_template`](MapData::default_from_template) to also initialize data.
    pub fn new(
        template: &MapPositions,
        data: impl Iterator<Item = (TilePos, T)>,
    ) -> MapResource<T> {
        let storage = MapResource::generate_storage(template, data);
        let neighbors = MapResource::generate_neighbors(&storage, template);

        MapResource { storage, neighbors }
    }

    /// Replace data at the specified position
    pub fn replace(&mut self, position: &TilePos, replace_with: T) {
        *(self.storage.get_mut(position).unwrap().get_mut()) = replace_with;
    }

    /// Get data stored at given position
    pub fn get(&self, position: &TilePos) -> Option<MapData<T>> {
        self.storage.get(position).cloned()
    }

    /// Get mutable access to data stored at given position
    pub fn get_mut(&mut self, position: &TilePos) -> Option<&mut MapData<T>> {
        self.storage.get_mut(position)
    }

    /// Get neighbor data for given position
    pub fn get_neighbors(&self, position: &TilePos) -> Option<&HexNeighbors<MapData<T>>> {
        self.neighbors.get(position)
    }

    /// Get mutable access to neighbor data for given position
    pub fn get_neighbors_mut(
        &mut self,
        position: &TilePos,
    ) -> Option<&mut HexNeighbors<MapData<T>>> {
        self.neighbors.get_mut(position)
    }

    /// Iterate over the positions managed by this resource
    pub fn positions(&self) -> impl Iterator<Item = &TilePos> {
        self.storage.keys()
    }

    /// Iterate over the data at all positions
    pub fn values(&self) -> impl Iterator<Item = &MapData<T>> {
        self.storage.values()
    }

    /// Iterate over the data at all positions, returning a mutable reference
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut MapData<T>> {
        self.storage.values_mut()
    }
}
