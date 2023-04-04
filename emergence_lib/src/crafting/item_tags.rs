//! Item tags are used to group items together for crafting recipes.
//!
//! Items can belong to multiple tags, and correspond to fields on [`ItemData`](crate::items::item_manifest::ItemData).

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A category of items.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
