//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    items::item_manifest::{Item, ItemManifest},
};

/// The item(s) that a unit is carrying.
#[derive(Component, Default, Clone, Debug, Deref, DerefMut)]
pub(crate) struct UnitInventory {
    /// The single item the unit is currently holding
    pub(crate) held_item: Option<Id<Item>>,
}

impl UnitInventory {
    /// Pretty foramtting for this type.
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        if let Some(item) = self.held_item {
            item_manifest.name(item).to_string()
        } else {
            "Nothing".to_string()
        }
    }
}
