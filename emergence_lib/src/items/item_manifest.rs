//! Defines write-only data for each variety of item.

use bevy::{
    reflect::{FromReflect, Reflect, TypeUuid},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::{loader::IsRawManifest, Id, Manifest},
    crafting::item_tags::{ItemKind, ItemTag},
};

/// The marker type for [`Id<Item>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Item;
/// Stores the read-only definitions for all items.
pub type ItemManifest = Manifest<Item, ItemData>;

impl ItemManifest {
    /// Does the provided `item_id` meet the requirements of the given `tag`?
    pub fn has_tag(&self, item_id: Id<Item>, tag: ItemTag) -> bool {
        let data = self.get(item_id);

        match tag {
            ItemTag::Compostable => data.compostable,
        }
    }

    /// Returns the complete list of tags that the given item belongs to.
    pub fn tags(&self, item_id: Id<Item>) -> Vec<ItemTag> {
        let data = self.get(item_id);

        let mut tags = Vec::new();

        if data.compostable {
            tags.push(ItemTag::Compostable);
        }

        tags
    }

    /// Returns the complete list of [`ItemKind`] that this item belongs to.
    pub fn kinds(&self, item_id: Id<Item>) -> Vec<ItemKind> {
        let mut kinds = Vec::new();
        kinds.push(ItemKind::Single(item_id));

        for tag in self.tags(item_id) {
            kinds.push(ItemKind::Tag(tag));
        }

        kinds
    }

    /// Returns the complete list of [`ItemKind`] that match the given `tag`.
    pub fn kinds_with_tag(&self, tag: ItemTag) -> Vec<ItemKind> {
        let mut kinds = Vec::new();

        for item_id in self.variants() {
            if self.has_tag(item_id, tag) {
                kinds.push(ItemKind::Single(item_id));
            }
        }

        kinds.push(ItemKind::Tag(tag));

        kinds
    }

    /// Returns the human-readable name associated with the provided `item_kind`.
    ///
    /// # Panics
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn name_of_kind(&self, item_kind: ItemKind) -> &str {
        match item_kind {
            ItemKind::Single(id) => self.name(id),
            ItemKind::Tag(tag) => tag.name(),
        }
    }
}

/// The data associated with each item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    /// The number of items that can fit in a single item slot.
    pub stack_size: u32,
    /// Can this item be composted?
    pub compostable: bool,
}

/// The [`ItemManifest`] as seen in the manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid, PartialEq)]
#[uuid = "cd9f4571-b0c4-4641-8d27-1c9c5ad4c812"]
pub struct RawItemManifest {
    /// The data for each item.
    pub items: HashMap<String, ItemData>,
}

impl IsRawManifest for RawItemManifest {
    const EXTENSION: &'static str = "item_manifest.json";

    type Marker = Item;
    type Data = ItemData;

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let mut manifest = Manifest::new();

        for (raw_id, raw_data) in self.items.clone() {
            // No additional preprocessing is needed.
            manifest.insert(raw_id, raw_data)
        }

        manifest
    }
}
