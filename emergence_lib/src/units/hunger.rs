//! Logic for finding and eating food when the [`EnergyPool`] is low.

use bevy::prelude::*;
use core::fmt::Display;
use leafwing_abilities::prelude::Pool;

use crate::{
    items::{ItemCount, ItemId},
    organisms::energy::{Energy, EnergyPool},
};

use super::{
    behavior::{CurrentAction, Goal, UnitAction},
    item_interaction::HeldItem,
};

/// The item(s) that a unit must consume to gain [`Energy`].
#[derive(Component, Clone, Debug)]
pub(crate) struct Diet {
    /// The item that must be eaten
    item: ItemId,
    /// The amount of energy restored per item destroyed
    energy: Energy,
}

impl Diet {
    /// Creates a new [`Diet`] component.
    pub(crate) fn new(item: ItemId, energy: Energy) -> Self {
        Diet { item, energy }
    }
}

impl Display for Diet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {} energy", self.item, self.energy)
    }
}

/// Swaps the goal to [`Goal::Eat`] when energy is low
pub(super) fn check_for_hunger(mut unit_query: Query<(&mut Goal, &EnergyPool, &Diet)>) {
    for (mut goal, energy_pool, diet) in unit_query.iter_mut() {
        if energy_pool.should_warn() {
            *goal = Goal::Eat(diet.item);
        }
    }
}

/// Swaps the goal to [`Goal::Eat`] when energy is low
pub(super) fn eat_held_items(
    mut unit_query: Query<(&CurrentAction, &mut HeldItem, &Diet, &mut EnergyPool)>,
) {
    for (current_action, mut held_item, diet, mut energy_pool) in unit_query.iter_mut() {
        if let UnitAction::Eat = current_action.action() {
            let item_count = ItemCount::new(diet.item, 1);
            let consumption_result = held_item.remove_item_all_or_nothing(&item_count);

            match consumption_result {
                Ok(_) => {
                    let proposed = energy_pool.current() + diet.energy;
                    energy_pool.set_current(proposed);
                }
                Err(error) => {
                    error!("{error:?}: unit tried to eat the wrong thing!")
                }
            }
        }
    }
}
