//! The raw manifest data before it has been processed.
//!
//! The processing will primarily remove the string IDs and replace them by numbers.

use bevy::{reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

/// A utility trait to ensure that all trait bounds are satisfied.
pub trait RawManifest: std::fmt::Debug + TypeUuid + Send + Sync + 'static {
    /// The path of the asset.
    fn path() -> &'static str;
}

/// The item data as seen in the original manifest file.
///
/// This will be converted to [`crate::items::ItemData`].
#[derive(Debug, Clone, Deserialize)]
pub struct RawItemData {
    /// The maximum number of items that can fit in a stack.
    stack_size: usize,
}

/// The item manifest as seen in the manifest file.
#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "cd9f4571-b0c4-4641-8d27-1c9c5ad4c812"]
pub struct RawItemManifest {
    /// The data for each item.
    items: HashMap<String, RawItemData>,
}

impl RawManifest for RawItemManifest {
    fn path() -> &'static str {
        "manifests/items.manifest.json"
    }
}
