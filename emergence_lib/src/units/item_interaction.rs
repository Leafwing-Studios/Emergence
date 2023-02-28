//! Holding, using and carrying items.

use bevy::prelude::*;

use crate::asset_management::manifest::{Id, Item};
use core::fmt::Display;

/// The item(s) that a unit is carrying.
#[derive(Component, Default, Clone, Debug, Deref, DerefMut)]
pub(crate) struct UnitInventory {
    /// The single item the unit is currently holding
    pub(crate) held_item: Option<Id<Item>>,
}

impl Display for UnitInventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(item) = self.held_item {
            write!(f, "{item}")
        } else {
            write!(f, "Nothing")
        }
    }
}
