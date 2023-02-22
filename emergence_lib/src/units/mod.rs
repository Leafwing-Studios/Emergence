//! Units are organisms that can move freely.

use crate::simulation::geometry::TilePos;
use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;
use core::fmt::Display;

use self::{
    behavior::{CurrentAction, Goal},
    item_interaction::HeldItem,
};

use crate::organisms::OrganismBundle;

pub(crate) mod behavior;
pub(crate) mod item_interaction;
mod movement;

/// The unique, string-based identifier of a unit.
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub(crate) struct UnitId {
    /// The unique identifier for this variety of unit.
    pub(crate) id: &'static str,
}

impl Display for UnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// An organism that can move around freely.
#[derive(Bundle)]
pub(crate) struct UnitBundle {
    /// Marker component.
    id: UnitId,
    /// The tile the unit is above.
    tile_pos: TilePos,
    /// What is the unit working towards.
    current_goal: Goal,
    /// What is the unit currently doing.
    current_action: CurrentAction,
    /// What is the unit currently holding, if anything?
    held_item: HeldItem,
    /// Organism data
    organism_bundle: OrganismBundle,
    /// Makes units pickable
    raycast_mesh: RaycastMesh<UnitId>,
}

impl UnitBundle {
    /// Initializes a new unit
    pub(crate) fn new(id: &'static str, tile_pos: TilePos) -> Self {
        UnitBundle {
            id: UnitId { id },
            tile_pos,
            current_goal: Goal::default(),
            current_action: CurrentAction::default(),
            held_item: HeldItem::default(),
            organism_bundle: OrganismBundle::default(),
            raycast_mesh: RaycastMesh::default(),
        }
    }
}

/// System labels for unit behavior
#[derive(SystemLabel)]
pub(crate) enum UnitSystem {
    /// Advances the timer of all unit actions.
    AdvanceTimers,
    /// Carry out the chosen action
    Act,
    /// Perform any necessary cleanup
    Cleanup,
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
            .add_system(
                item_interaction::pickup_and_drop_items
                    .label(UnitSystem::Act)
                    .after(UnitSystem::AdvanceTimers),
            )
            .add_system(
                item_interaction::clear_empty_slots
                    .label(UnitSystem::Cleanup)
                    .after(UnitSystem::Act),
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
