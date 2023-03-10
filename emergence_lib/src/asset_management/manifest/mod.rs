//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

pub(crate) use self::emergence_markers::*;
pub(crate) use self::identifier::*;

mod emergence_markers;
mod identifier;

use bevy::{prelude::*, utils::HashMap};
use std::fmt::Debug;

/// Write-once data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource)]
pub(crate) struct Manifest<T, Data>
where
    T: 'static,
    Data: Debug,
{
    /// The internal mapping.
    map: HashMap<Id<T>, Data>,
}

impl<T, Data> Manifest<T, Data>
where
    Data: Debug,
{
    /// Create a new manifest with the given definitions.
    pub fn new(map: HashMap<Id<T>, Data>) -> Self {
        Self { map }
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: Id<T>) -> &Data {
        self.map
            .get(&id)
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = Id<T>> + '_ {
        self.map.keys().copied()
    }
}
