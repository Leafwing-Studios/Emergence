//! The raw manifest data before it has been processed.
//!
//! The processing will primarily remove the string IDs and replace them by numbers.

use bevy::{reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

use crate::items::ItemData;

use super::{Id, Item, Manifest};

/// A utility trait to ensure that all trait bounds are satisfied.
pub(crate) trait RawManifest:
    std::fmt::Debug + TypeUuid + Send + Sync + for<'de> Deserialize<'de> + 'static
{
    /// The marker type for the manifest ID.
    type Marker: 'static + Send + Sync;

    /// The type of the processed manifest data.
    type Data: std::fmt::Debug + Send + Sync;

    /// The path of the asset.
    fn path() -> &'static str;

    /// Process the raw manifest from the asset file to the manifest data used in-game.
    fn process(&self) -> Manifest<Self::Marker, Self::Data>;
}

/// The item data as seen in the original manifest file.
///
/// This will be converted to [`crate::items::ItemData`].
#[derive(Debug, Clone, Deserialize)]
pub struct RawItemData {
    /// The maximum number of items that can fit in a stack.
    stack_size: usize,
}

impl From<&RawItemData> for ItemData {
    fn from(value: &RawItemData) -> Self {
        Self::new(value.stack_size)
    }
}

/// The item manifest as seen in the manifest file.
#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "cd9f4571-b0c4-4641-8d27-1c9c5ad4c812"]
pub(crate) struct RawItemManifest {
    /// The data for each item.
    items: HashMap<String, RawItemData>,
}

impl RawManifest for RawItemManifest {
    type Marker = Item;
    type Data = ItemData;

    fn path() -> &'static str {
        "manifests/items.manifest.json"
    }

    fn process(&self) -> Manifest<Self::Marker, Self::Data> {
        let map: HashMap<Id<Self::Marker>, Self::Data> = self
            .items
            .iter()
            .map(|(str_id, raw_data)| {
                let id = Id::<Self::Marker>::from_string_id(str_id);
                let data = Self::Data::from(raw_data);

                (id, data)
            })
            .collect();

        Manifest { data_map: map }
    }
}
