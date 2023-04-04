//! Item tags are used to group items together for crafting recipes.
//!
//! Items can belong to multiple tags, and correspond to fields on [`ItemData`](crate::items::item_manifest::ItemData).

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::{
    asset_management::manifest::Id,
    items::item_manifest::{Item, ItemManifest},
};

/// A category of items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemTag {
    /// Items that can be composted.
    Compostable,
}

impl Display for ItemTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemTag::Compostable => write!(f, "Compostable"),
        }
    }
}

/// An item or collection of items that shares a property.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemKind {
    /// Exactly one type of item.
    Single(Id<Item>),
    /// Any item that matches the given tag.
    Tag(ItemTag),
}

impl ItemKind {
    /// Returns true if the given item matches this kind.
    pub fn matches(&self, item_id: Id<Item>, item_manifest: &ItemManifest) -> bool {
        match self {
            ItemKind::Single(id) => *id == item_id,
            ItemKind::Tag(tag) => item_manifest.has_tag(item_id, *tag),
        }
    }
}
