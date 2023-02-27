//! Logic for finding and eating food when the [`EnergyPool`] is low.

use bevy::prelude::*;
use core::fmt::Display;

use crate::{
    items::ItemId,
    organisms::energy::{Energy, EnergyPool},
};

use super::goals::Goal;

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

    /// The type of item that this unit must consume.
    pub(crate) fn item(&self) -> ItemId {
        self.item
    }

    /// The amount of [`Energy`] gained when a single item of the correct type is consumed.
    pub(crate) fn energy(&self) -> Energy {
        self.energy
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
        if energy_pool.is_hungry() {
            *goal = Goal::Eat(diet.item);
        } else if matches!(*goal, Goal::Eat(..)) && energy_pool.is_satiated() {
            *goal = Goal::Wander
        }
    }
}
