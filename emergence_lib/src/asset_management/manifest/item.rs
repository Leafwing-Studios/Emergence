use bevy::reflect::{FromReflect, Reflect};

use crate::items::ItemData;

use super::Manifest;

/// The marker type for [`Id<Item>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Item;
/// Stores the read-only definitions for all items.
pub(crate) type ItemManifest = Manifest<Item, ItemData>;
