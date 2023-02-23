//! Logic for finding and eating food when the [`EnergyPool`] is low.

use bevy::prelude::*;
use core::fmt::Display;

use crate::{
    items::ItemId,
    organisms::energy::{Energy, EnergyPool},
};

use super::behavior::Goal;

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
