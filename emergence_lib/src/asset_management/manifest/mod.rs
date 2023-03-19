//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

pub use self::emergence_markers::*;
pub use self::identifier::*;

mod emergence_markers;
mod identifier;
mod loader;
pub(crate) mod plugin;
mod raw;

use bevy::{prelude::*, utils::HashMap};
use std::fmt::Debug;

/// Write-only data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource)]
pub(crate) struct Manifest<T, Data>
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

    /// Adds an entry to the manifest by supplying the `name` associated with the [`Id`] type to be constructed.
    ///
    /// Returns any existing `Data` entry if this overwrote the data.
    pub fn insert(&mut self, name: &str, data: Data) {
        let id = Id::from_string_id(name);

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
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
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
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = Id<T>> + '_ {
        self.data_map.keys().copied()
    }
}
