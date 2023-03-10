//! Code for a generic identifier type

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
    pub(crate) const fn new(str: &'static str) -> Id<T> {
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
