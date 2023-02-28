//! Units are organisms that can move freely.

use crate::{
    asset_management::{
        manifest::{Id, Unit},
        units::UnitHandles,
    },
    organisms::energy::EnergyPool,
    simulation::geometry::{Facing, MapGeometry, TilePos},
};
use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use self::{actions::CurrentAction, goals::Goal, hunger::Diet, item_interaction::HeldItem};

use crate::organisms::OrganismBundle;

pub(crate) mod actions;
pub(crate) mod goals;
pub(crate) mod hunger;
pub(crate) mod item_interaction;
mod reproduction;

impl Id<Unit> {
    // TODO: read these from disk
    /// The id of an ant
    pub(crate) fn ant() -> Self {
        Self::new("ant")
    }
}

/// An organism that can move around freely.
#[derive(Bundle)]
pub(crate) struct UnitBundle {
    /// Marker component.
    unit_id: Id<Unit>,
    /// The tile the unit is above.
    tile_pos: TilePos,
    /// The direction that the unit is facing.
    facing: Facing,
    /// What is the unit working towards.
    current_goal: Goal,
    /// What is the unit currently doing.
    current_action: CurrentAction,
    /// What is the unit currently holding, if anything?
    held_item: HeldItem,
    /// What does this unit need to eat?
    diet: Diet,
    /// Organism data
    organism_bundle: OrganismBundle,
    /// Makes units pickable
    raycast_mesh: RaycastMesh<Id<Unit>>,
    /// The mesh used for raycasting
    mesh: Handle<Mesh>,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl UnitBundle {
    /// Initializes a new unit
    // TODO: use a UnitManifest
    pub(crate) fn new(
        unit_id: Id<Unit>,
        tile_pos: TilePos,
        energy_pool: EnergyPool,
        diet: Diet,
        unit_handles: &UnitHandles,
        map_geometry: &MapGeometry,
    ) -> Self {
        let scene_handle = unit_handles.scenes.get(&unit_id).unwrap();

        UnitBundle {
            unit_id,
            tile_pos,
            facing: Facing::default(),
            current_goal: Goal::default(),
            current_action: CurrentAction::default(),
            held_item: HeldItem::default(),
            diet,
            organism_bundle: OrganismBundle::new(energy_pool),
            raycast_mesh: RaycastMesh::default(),
            mesh: unit_handles.picking_mesh.clone_weak(),
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(tile_pos.into_world_pos(map_geometry)),
                ..default()
            },
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
        app.add_system(actions::advance_action_timer.label(UnitSystem::AdvanceTimers))
            .add_system(
                actions::handle_actions
                    .label(UnitSystem::Act)
                    .after(UnitSystem::AdvanceTimers),
            )
            .add_system(
                item_interaction::clear_empty_slots
                    .label(UnitSystem::Cleanup)
                    .after(UnitSystem::Act),
            )
            .add_system(goals::choose_goal.label(UnitSystem::ChooseGoal))
            .add_system(
                actions::choose_actions
                    .label(UnitSystem::ChooseNewAction)
                    .after(UnitSystem::Act)
                    .after(UnitSystem::ChooseGoal),
            )
            .add_system(reproduction::hatch_ant_eggs)
            .add_system(hunger::check_for_hunger.before(UnitSystem::ChooseNewAction));
    }
}
