//! Read-only definitions for entities.
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Read-only data definitions.
#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct Manifest<Id, Data>(HashMap<Id, Data>)
where
    Id: Debug + PartialEq + Eq + Hash,
    Data: Debug;

impl<Id, Data> Manifest<Id, Data>
where
    Id: Debug + Display + PartialEq + Eq + Hash,
    Data: Debug,
{
    /// Create a new manifest with the given definitions.
    pub fn new(map: HashMap<Id, Data>) -> Self {
        Self(map)
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: &Id) -> &Data {
        self.0
            .get(id)
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
    }
}
