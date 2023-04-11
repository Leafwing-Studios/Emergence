//! Logic for finding and eating food when the [`EnergyPool`] is low.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    crafting::item_tags::ItemKind,
    items::item_manifest::{Item, ItemManifest},
    organisms::energy::{Energy, EnergyPool},
};

use super::{
    goals::Goal,
    item_interaction::UnitInventory,
    unit_manifest::{Unit, UnitManifest},
};

/// The item(s) that a unit must consume to gain [`Energy`].
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Diet {
    /// The item that must be eaten
    item_kind: ItemKind,
    /// The amount of energy restored per item destroyed
    energy: Energy,
}

/// The unprocessed equivalent of [`Diet`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawDiet {
    /// The item that must be eaten
    item: String,
    /// The amount of energy restored per item destroyed
    energy: Energy,
}

impl RawDiet {
    /// Creates a new [`RawDiet`].
    pub fn new(item: &str, energy: f32) -> Self {
        Self {
            item: item.to_string(),
            energy: Energy(energy),
        }
    }
}

impl From<RawDiet> for Diet {
    fn from(raw_diet: RawDiet) -> Self {
        Diet {
            item_kind: ItemKind::Single(Id::from_name(raw_diet.item)),
            energy: raw_diet.energy,
        }
    }
}

impl Diet {
    /// Creates a new [`Diet`] component.
    pub fn new(item: Id<Item>, energy: Energy) -> Self {
        Diet {
            item_kind: ItemKind::Single(item),
            energy,
        }
    }

    /// The kind of item that this unit must consume.
    pub(super) fn item_kind(&self) -> ItemKind {
        self.item_kind
    }

    /// The amount of [`Energy`] gained when a single item of the correct type is consumed.
    pub(super) fn energy(&self) -> Energy {
        self.energy
    }

    /// Pretty formatting for this type
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        format!(
            "{} -> {} energy",
            item_manifest.name_of_kind(self.item_kind),
            self.energy
        )
    }
}

/// Swaps the goal to [`Goal::Eat`] when energy is low
pub(super) fn check_for_hunger(
    mut unit_query: Query<(&mut Goal, &EnergyPool, &Id<Unit>, &UnitInventory)>,
    unit_manifest: Res<UnitManifest>,
) {
    for (mut goal, energy_pool, unit_id, unit_inventory) in unit_query.iter_mut() {
        if energy_pool.is_hungry() {
            // Make sure to put down any item we're holding before eating
            if let Some(item) = unit_inventory.held_item {
                if *goal == Goal::Store(ItemKind::Single(item)) {
                    continue;
                };
            }

            let diet = &unit_manifest.get(*unit_id).diet;
            *goal = Goal::Eat(diet.item_kind);
        } else if matches!(*goal, Goal::Eat(..)) && energy_pool.is_satiated() {
            *goal = Goal::Wander {
                remaining_actions: None,
            }
        }
    }
}
