//! Read-only definitions for entities.
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

/// Read-only data definitions.
#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct Manifest<Id, Data>(HashMap<Id, Data>)
where
    Id: Debug + PartialEq + Eq + Hash,
    Data: Debug;

impl<Id, Data> Manifest<Id, Data>
where
    Id: Debug + PartialEq + Eq + Hash,
    Data: Debug,
{
    /// Create a new manifest with the given definitions.
    pub fn new(map: HashMap<Id, Data>) -> Self {
        Self(map)
    }

    /// Get the data entry for the given ID.
    pub fn get(&self, id: &Id) -> Option<&Data> {
        self.0.get(id)
    }
}
