//! Code for managing data that is deeply tied to the map

use crate::simulation::map::hex_patch::HexPatch;
use crate::simulation::map::MapPositions;
use bevy::prelude::Resource;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Spatial data for use with the [`MapIndex`] struct.
#[derive(Debug)]
pub struct MapData<T> {
    /// The `Arc` allows for multiple references to the data, the `RwLock` allows for
    /// multiple readers/single-writer manipulation of the data.
    pub(crate) inner: Arc<RwLock<T>>,
}

// We cannot derive Clone as it would force a `Clone` bound on `T`.
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

    /// Replace internal data
    pub fn replace(&mut self, new_data: T) {
        *self.get_mut() = new_data;
    }
}

/// An acceleration data structure for looking up game data that is tied to a fixed [`TilePos`].
///
/// It can give you [`MapData<T>`](MapData) at a given tile position, or it can give you
/// [`Hexpatches<MapData<T>>`](HexPatch) for the given position.
///
/// Internally, [`MapData`] is stored in a [`HashMap`] for each position, in the `storage` field,
/// and this same data is then referenced by the `patches` field.
#[derive(Resource)]
pub struct MapIndex<T> {
    /// Primary internal storage of data associated with each position
    pub(crate) storage: HashMap<TilePos, MapData<T>>,
    /// [`HexPatch`] of data centered at each position
    ///
    /// It is based off of references to data in `storage`.
    pub(crate) patches: HashMap<TilePos, HexPatch<MapData<T>>>,
}

impl<T> MapIndex<T>
where
    T: Default,
{
    /// Create new from an underlying [`MapPositions`] template
    ///
    /// This allocates capacity and initializes patches based on the template provided.
    ///
    /// This method only exists when `T` implements [`Default`].
    pub fn default_from_template(template: &MapPositions) -> MapIndex<T> {
        let storage = MapIndex::generate_storage(
            template,
            template
                .iter_positions()
                .map(|position| (*position, T::default())),
        );

        let patches = MapIndex::generate_patches(&storage, template);

        MapIndex { storage, patches }
    }
}

impl<T> MapIndex<T> {
    /// Generate the storage [`HashMap`]
    pub fn generate_storage(
        template: &MapPositions,
        data: impl Iterator<Item = (TilePos, T)>,
    ) -> HashMap<TilePos, MapData<T>> {
        let mut storage = HashMap::with_capacity(template.n_positions());
        storage.extend(data.map(|(tile_pos, t)| (tile_pos, MapData::new(t))));
        storage
    }

    /// Generate patches, recording the data of type `T` in each adjacent cell
    pub fn generate_patches(
        storage: &HashMap<TilePos, MapData<T>>,
        template: &MapPositions,
    ) -> HashMap<TilePos, HexPatch<MapData<T>>> {
        let mut patches = HashMap::with_capacity(template.n_positions());
        patches.extend(template.iter_positions().filter_map(|position| {
            let tile_patch = template.get_patch(position)?;
            let data_patch = tile_patch.and_then_ref(|position| {
                let map_data = storage.get(position)?;
                let map_data_clone: MapData<T> = map_data.clone();
                Some(map_data_clone)
            });
            Some((*position, data_patch))
        }));
        patches
    }

    /// Create new from an underlying [`MapPositions`] template.
    ///
    /// This allocates capacity and initializes patches based on the template provided.
    ///
    /// If your underlying data implements [`Default`], you could use
    /// [`default_from_template`](MapIndex::default_from_template) to also initialize data.
    pub fn new(template: &MapPositions, data: impl Iterator<Item = (TilePos, T)>) -> MapIndex<T> {
        let storage = MapIndex::generate_storage(template, data);
        let patches = MapIndex::generate_patches(&storage, template);

        MapIndex { storage, patches }
    }

    /// Update data for given tile positions
    pub fn update(&mut self, new_data: impl Iterator<Item = (TilePos, T)>) {
        new_data.for_each(|(position, data)| {
            if let Some(map_data) = self.storage.get_mut(&position) {
                map_data.replace(data);
            }
        });
    }

    /// Replace data at the specified position
    pub fn replace(&mut self, position: &TilePos, replace_with: T) {
        *(self.storage.get_mut(position).unwrap().get_mut()) = replace_with;
    }

    /// Get data stored at `position`
    pub fn get(&self, position: &TilePos) -> Option<MapData<T>> {
        self.storage.get(position).cloned()
    }

    /// Get mutable access to data stored at `position`
    pub fn get_mut(&mut self, position: &TilePos) -> Option<&mut MapData<T>> {
        self.storage.get_mut(position)
    }

    /// Get immutable access to the [`HexPatch`] of adjacent data around `position`
    pub fn get_patch(&self, position: &TilePos) -> Option<&HexPatch<MapData<T>>> {
        self.patches.get(position)
    }

    /// Get mutable access to [`HexPatch`] of adjacent data around `position`
    pub fn get_patch_mut(&mut self, position: &TilePos) -> Option<&mut HexPatch<MapData<T>>> {
        self.patches.get_mut(position)
    }

    /// Iterate over the positions managed by this resource
    pub fn positions(&self) -> impl Iterator<Item = &TilePos> {
        self.storage.keys()
    }

    /// Iterate over the data stored at each position, returning a shared reference
    pub fn values(&self) -> impl Iterator<Item = &MapData<T>> {
        self.storage.values()
    }

    /// Iterate over the data stored at each position, returning a mutable reference
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut MapData<T>> {
        self.storage.values_mut()
    }
}
