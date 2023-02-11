//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Write-once data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct Manifest<Id, Data>
where
    Id: Debug + PartialEq + Eq + Hash,
    Data: Debug,
{
    map: HashMap<Id, Data>,
}

impl<Id, Data> Manifest<Id, Data>
where
    Id: Debug + Display + PartialEq + Eq + Hash,
    Data: Debug,
{
    /// Create a new manifest with the given definitions.
    pub fn new(map: HashMap<Id, Data>) -> Self {
        Self { map }
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: &Id) -> &Data {
        self.map
            .get(id)
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = &Id> {
        self.map.keys()
    }
}
