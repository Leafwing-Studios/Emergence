//! Code for a generic identifier type

use bevy::{prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// The unique identifier of type `T`.
///
/// This is tiny [`Copy`] type, used to quickly and uniquely identify game objects.
/// Unlike enum variants, these can be read from disk and constructed at runtime.
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
    pub fn from_name(name: String) -> Self {
        // Algorithm adopted from <https://cp-algorithms.com/string/string-hashing.html>

        let mut value = 0;
        let mut p_pow = 1;

        name.bytes().for_each(|byte| {
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
        Some(self.cmp(other))
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
        *self
    }
}

impl<T> Copy for Id<T> {}
