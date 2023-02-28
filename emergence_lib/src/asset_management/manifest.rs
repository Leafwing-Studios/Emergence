//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

pub(crate) use self::emergence_markers::*;
pub(crate) use self::identifier::*;

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

/// Code for a generic identifier type
mod identifier {
    use bevy::prelude::Component;
    use serde::{Deserialize, Serialize};
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
        marker::PhantomData,
    };

    /// The unique identifier of type `T`.
    ///
    /// This is tiny [`Copy`] type, used to quickly and uniquely identify game objects.
    /// Unlike enum variants, these can be read from disk and constructred at runtime.
    ///
    /// It can be stored as a component to identify the variety of game object used.
    #[derive(Component, Serialize, Deserialize)]
    pub(crate) struct Id<T> {
        /// The internal string
        str: &'static str,
        /// Marker to make the compiler happy
        _phantom: PhantomData<T>,
    }

    impl<T> Id<T> {
        /// Creates a new identifier from a static-lifetime string.
        pub(crate) fn new(str: &'static str) -> Id<T> {
            Id {
                str,
                _phantom: PhantomData,
            }
        }
    }

    impl<T> Debug for Id<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Id").field("str", &self.str).finish()
        }
    }

    impl<T> PartialEq for Id<T> {
        fn eq(&self, other: &Self) -> bool {
            self.str == other.str
        }
    }

    impl<T> Eq for Id<T> {}

    impl<T> PartialOrd for Id<T> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.str.partial_cmp(other.str)
        }
    }

    impl<T> Ord for Id<T> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.str.cmp(other.str)
        }
    }

    impl<T> Hash for Id<T> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.str.hash(state);
        }
    }

    impl<T> Clone for Id<T> {
        fn clone(&self) -> Self {
            Self {
                str: self.str,
                _phantom: PhantomData,
            }
        }
    }

    impl<T> Copy for Id<T> {}

    impl<T> Display for Id<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.str)
        }
    }
}

/// Code for Emergence-specific marker types
mod emergence_markers {
    use crate::{
        items::{recipe::RecipeData, ItemData},
        structures::StructureData,
        units::UnitData,
    };

    use super::Manifest;

    /// The marker type for [`Id<Recipe>`](super::Id).
    pub(crate) struct Recipe;

    /// Stores the read-only definitions for all recipes.
    pub(crate) type RecipeManifest = Manifest<Recipe, RecipeData>;

    /// The marker type for [`Id<Unit>`](super::Id).
    pub(crate) struct Unit;
    /// Stores the read-only definitions for all units.
    pub(crate) type UnitManifest = Manifest<Unit, UnitData>;

    /// The marker type for [`Id<Structure>`](super::Id).
    pub(crate) struct Structure;
    /// Stores the read-only definitions for all structures.
    pub(crate) type StructureManifest = Manifest<Structure, StructureData>;

    /// The marker type for [`Id<Item>`](super::Id).
    pub(crate) struct Item;
    /// Stores the read-only definitions for all items.
    pub(crate) type ItemManifest = Manifest<Item, ItemData>;
}
