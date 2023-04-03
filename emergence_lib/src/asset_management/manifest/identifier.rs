//! Code for a generic identifier type

use bevy::{prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// The unique identifier of type `T`.
///
/// This is tiny [`Copy`] type, used to quickly and uniquely identify game objects.
/// Unlike enum variants, these can be read from disk and constructred at runtime.
///
/// It can be stored as a component to identify the variety of game object used.
#[derive(Component, Reflect, Serialize, Deserialize)]
pub struct Id<T> {
    /// The unique identifier.
    ///
    /// This is usually the hash of a string identifier used in the manifest files.
    /// The number value is used to handle the data more efficiently in the game.
    value: u64,

    /// Marker to make the compiler happy
    #[reflect(ignore)]
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// An unprocessed [`Id`] that stores the string identifier.
#[derive(Reflect, Serialize, Deserialize)]
pub struct RawId<T> {
    /// The string identifier.
    ///
    /// This is used to create the [`Id`] when the manifest is loaded.
    name: String,

    /// Marker to make the compiler happy
    #[reflect(ignore)]
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> RawId<T> {
    /// Creates a new raw ID from the given string.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name,
            _phantom: PhantomData,
        }
    }

    /// Gets the string identifier of this ID.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> From<RawId<T>> for Id<T> {
    fn from(raw: RawId<T>) -> Self {
        Self::from_name(&raw.name)
    }
}

impl<T> Debug for RawId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RawId").field(&self.name).finish()
    }
}

impl<T> Clone for RawId<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for RawId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T> Eq for RawId<T> {}

impl<T> Hash for RawId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// A constant used in the hashing algorithm of the IDs.
///
/// This should be a positive prime number, roughly equal to the number of characters in the input alphabet.
const HASH_P: u64 = 53;

/// A constant used in the hashing algorithm of the IDs.
///
/// This should be a large prime number as it is used for modulo operations.
/// Larger numbers have a lower chance of a hash collision.
const HASH_M: u64 = 1_000_000_009;

impl<T> Id<T> {
    /// Create a new identifier from the given unique number.
    const fn new(value: u64) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// Creates a new ID from human-readable string identifier.
    ///
    /// This ID is created as a hash of the string.
    pub fn from_name(str: &str) -> Self {
        // Algorithm adopted from <https://cp-algorithms.com/string/string-hashing.html>

        let mut value = 0;
        let mut p_pow = 1;

        str.bytes().for_each(|byte| {
            value = (value + (byte as u64 + 1) * p_pow) % HASH_M;
            p_pow = (p_pow * HASH_P) % HASH_M;
        });

        Self::new(value)
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Id").field("value", &self.value).finish()
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}
