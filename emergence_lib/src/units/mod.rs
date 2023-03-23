//! Units are organisms that can move freely.

use crate::{
    asset_management::{
        manifest::{Id, Manifest, Unit, UnitManifest},
        units::UnitHandles,
    },
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::Lifecycle,
        OrganismId, OrganismVariety,
    },
    player_interaction::InteractionSystem,
    simulation::{
        geometry::{Facing, MapGeometry, TilePos},
        SimulationSet,
    },
};
use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;
use leafwing_abilities::prelude::Pool;
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng};

use self::{
    actions::CurrentAction, goals::Goal, hunger::Diet, impatience::ImpatiencePool,
    item_interaction::UnitInventory,
};

use crate::organisms::OrganismBundle;

pub(crate) mod actions;
pub(crate) mod goals;
pub(crate) mod hunger;
pub(crate) mod impatience;
pub(crate) mod item_interaction;
mod reproduction;

/// The data associated with each variety of unit
#[derive(Debug, Clone)]
pub(crate) struct UnitData {
    /// The data shared by all organisms
    organism_variety: OrganismVariety,
    /// What this unit type needs to eat
    diet: Diet,
    /// How much impatience this unit can accumulate before getting too frustrated and picking a new task.
    max_impatience: u8,
    /// How many actions will units of this type take while wandering before picking a new goal?
    ///
    /// This stores a [`WeightedIndex`] to allow for multimodal distributions.
    wandering_behavior: WanderingBehavior,
}

#[derive(Debug, Clone)]
struct WanderingBehavior {
    /// How many actions will units take while wandering before picking a new goal?
    wander_durations: Vec<u16>,
    /// The relative probability of each value in `mean_free_wander_period`.
    weights: WeightedIndex<f32>,
}

impl WanderingBehavior {
    fn sample(&self, rng: &mut ThreadRng) -> u16 {
        self.wander_durations[self.weights.sample(rng)]
    }
}

impl FromIterator<(u16, f32)> for WanderingBehavior {
    fn from_iter<T: IntoIterator<Item = (u16, f32)>>(iter: T) -> Self {
        let mut wander_durations = Vec::new();
        let mut weights = Vec::new();
        for (duration, weight) in iter {
            wander_durations.push(duration);
            weights.push(weight);
        }
        WanderingBehavior {
            wander_durations,
            weights: WeightedIndex::new(weights).unwrap(),
        }
    }
}

impl UnitData {
    /// Returns the [`OrganismVariety`] data for this type of unit.
    pub(crate) fn organism_variety(&self) -> &OrganismVariety {
        &self.organism_variety
    }
}

impl Default for UnitManifest {
    fn default() -> Self {
        let mut manifest: UnitManifest = Manifest::new();

        // TODO: load this from disk
        manifest.insert(
            "ant",
            UnitData {
                organism_variety: OrganismVariety {
                    prototypical_form: OrganismId::Unit(Id::from_name("ant")),
                    lifecycle: Lifecycle::STATIC,
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                },
                diet: Diet::new(Id::from_name("leuco_chunk"), Energy(50.)),
                max_impatience: 10,
                wandering_behavior: WanderingBehavior::from_iter([
                    (0, 0.7),
                    (64, 0.2),
                    (1024, 0.1),
                ]),
            },
        );

        manifest
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
    /// How frustrated this unit is.
    ///
    /// When full, the current goal will be abandoned.
    impatience: ImpatiencePool,
    /// What is the unit currently doing.
    current_action: CurrentAction,
    /// What is the unit currently holding, if anything?
    held_item: UnitInventory,
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
        unit_data: UnitData,
        unit_handles: &UnitHandles,
        map_geometry: &MapGeometry,
    ) -> Self {
        let scene_handle = unit_handles.scenes.get(&unit_id).unwrap();

        UnitBundle {
            unit_id,
            tile_pos,
            facing: Facing::default(),
            current_goal: Goal::default(),
            impatience: ImpatiencePool::new(unit_data.max_impatience),
            current_action: CurrentAction::default(),
            held_item: UnitInventory::default(),
            diet: unit_data.diet,
            organism_bundle: OrganismBundle::new(
                unit_data.organism_variety.energy_pool,
                unit_data.organism_variety.lifecycle,
            ),
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

/// System sets for unit behavior
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum UnitSystem {
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
        app.init_resource::<UnitManifest>().add_systems(
            (
                actions::advance_action_timer.in_set(UnitSystem::AdvanceTimers),
                actions::start_actions
                    .in_set(UnitSystem::Act)
                    .before(actions::finish_actions),
                actions::finish_actions
                    .in_set(UnitSystem::Act)
                    .after(UnitSystem::AdvanceTimers)
                    // This must occur after MarkedForDemolition is added,
                    // or we'll get a panic due to inserting a component on a despawned entity
                    .after(InteractionSystem::ManagePreviews),
                goals::choose_goal.in_set(UnitSystem::ChooseGoal),
                actions::choose_actions
                    .in_set(UnitSystem::ChooseNewAction)
                    .after(UnitSystem::Act)
                    .after(UnitSystem::ChooseGoal),
                reproduction::hatch_ant_eggs,
                hunger::check_for_hunger.before(UnitSystem::ChooseNewAction),
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
