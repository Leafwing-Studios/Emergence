//! Graphics and animation code for units.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    units::{item_interaction::UnitInventory, unit_manifest::Unit},
};

/// Shows the item that each unit is holding
pub(super) fn display_held_item(
    unit_query: Query<&UnitInventory, (With<Id<Unit>>, Changed<UnitInventory>)>,
) {
    for _held_item in unit_query.iter() {
        // TODO: actually display this
    }
}
