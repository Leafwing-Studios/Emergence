//! Graphics and animation code for units.

use bevy::prelude::*;

use crate::units::{item_interaction::HeldItem, UnitId};

/// Shows the item that each unit is holding
pub(super) fn display_held_item(unit_query: Query<&HeldItem, (With<UnitId>, Changed<HeldItem>)>) {
    for _held_item in unit_query.iter() {
        // TODO: actually display this
    }
}
