//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

mod identifier;
pub use self::identifier::*;
mod loader;
pub(crate) mod plugin;
mod raw;
pub(crate) mod terrain;
pub use terrain::*;
pub(crate) mod item;
pub use item::*;
pub(crate) mod recipe;
pub use recipe::*;

use bevy::{prelude::*, utils::HashMap};
use std::fmt::Debug;

/// Write-only data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource)]
pub struct Manifest<T, Data>
where
    T: 'static,
    Data: Debug,
{
    /// The internal mapping to the data
    data_map: HashMap<Id<T>, Data>,

    /// The human-readable name associated with each Id.
    name_map: HashMap<Id<T>, String>,
}

impl<T, Data> Manifest<T, Data>
where
    Data: Debug,
{
    /// Create a new empty manifest.
    pub fn new() -> Self {
        Self {
            data_map: HashMap::default(),
            name_map: HashMap::default(),
        }
    }

    /// Returns a reference to the internal data map.
    pub fn data_map(&self) -> &HashMap<Id<T>, Data> {
        &self.data_map
    }

    /// Returns a reference to the internal name map.
    pub fn name_map(&self) -> &HashMap<Id<T>, String> {
        &self.name_map
    }

    /// Adds an entry to the manifest by supplying the `name` associated with the [`Id`] type to be constructed.
    ///
    /// Returns any existing `Data` entry if this overwrote the data.
    pub fn insert(&mut self, name: &str, data: Data) {
        let id = Id::from_name(name);

        self.data_map.insert(id, data);
        self.name_map.insert(id, name.to_string());
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: Id<T>) -> &Data {
        self.data_map
            .get(&id)
            .unwrap_or_else(|| panic!("ID {id:?} not found in manifest"))
    }

    /// Returns the human-readable name associated with the provided `id`.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn name(&self, id: Id<T>) -> &str {
        self.name_map
            .get(&id)
            .unwrap_or_else(|| panic!("ID {id:?} not found in manifest"))
    }

    /// Returns the complete list of names of the loaded options.
    ///
    /// The order is arbitrary.
    pub fn names(&self) -> impl IntoIterator<Item = &str> {
        let variants = self.variants();
        variants.into_iter().map(|id| self.name(id))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = Id<T>> + '_ {
        self.data_map.keys().copied()
    }
}
