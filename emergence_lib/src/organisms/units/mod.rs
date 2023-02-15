//! Units are organisms that can move freely.

use crate::simulation::geometry::TilePos;
use bevy::prelude::*;

use self::behavior::{CurrentAction, CurrentGoal};

use super::OrganismBundle;

mod behavior;
mod movement;

/// The unique, string-based identifier of a unit.
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct UnitId {
    /// The unique identifier for this variety of unit.
    pub(crate) id: &'static str,
}

/// An organism that can move around freely.
#[derive(Bundle)]
pub(crate) struct UnitBundle {
    /// Marker component.
    id: UnitId,
    /// The tile the unit is above.
    tile_pos: TilePos,
    /// What is the unit working towards.
    current_goal: CurrentGoal,
    /// What is the unit currently doing.
    current_action: CurrentAction,
    /// Organism data
    organism_bundle: OrganismBundle,
}

impl UnitBundle {
    /// Initializes a new unit
    pub(crate) fn new(id: &'static str, tile_pos: TilePos) -> Self {
        UnitBundle {
            id: UnitId { id },
            tile_pos,
            current_goal: CurrentGoal::default(),
            current_action: CurrentAction::default(),
            organism_bundle: OrganismBundle::default(),
        }
    }
}

/// System labels for unit behavior
#[derive(SystemLabel)]
pub enum UnitSystem {
    /// Advances the timer of all unit actions.
    AdvanceTimers,
    /// Carry out the chosen action
    Act,
    /// Pick a higher level goal to pursue
    ChooseGoal,
    /// Pick an action that will get the agent closer to the goal being pursued
    ChooseNewAction,
}

/// Contains unit behavior
pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(behavior::advance_action_timer.label(UnitSystem::AdvanceTimers))
            .add_system(
                movement::move_units
                    .label(UnitSystem::Act)
                    .after(UnitSystem::AdvanceTimers),
            )
            .add_system(behavior::choose_goal.label(UnitSystem::ChooseGoal))
            .add_system(
                behavior::choose_actions
                    .label(UnitSystem::ChooseNewAction)
                    .after(UnitSystem::Act)
                    .after(UnitSystem::ChooseGoal),
            );
    }
}
