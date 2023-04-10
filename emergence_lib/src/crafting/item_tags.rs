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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemTag {
    /// Items that can be composted.
    Compostable,
    /// Items that will grow into something if left on the ground.
    Seed,
}

impl ItemTag {
    /// Returns the human-readable name associated with this tag.
    pub fn name(&self) -> &'static str {
        match self {
            ItemTag::Compostable => "Compostable",
            ItemTag::Seed => "Seed",
        }
    }
}

impl Display for ItemTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// An item or collection of items that shares a property.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
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

    /// Returns true if this kind is compatible with the provided tag.
    pub fn is_compatible_with(&self, tag: ItemTag, item_manifest: &ItemManifest) -> bool {
        match self {
            ItemKind::Single(item_id) => item_manifest.has_tag(*item_id, tag),
            ItemKind::Tag(self_tag) => *self_tag == tag,
        }
    }
}
