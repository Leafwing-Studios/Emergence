//! Units are organisms that can move freely.

use crate::{
    asset_management::{
        manifest::{Id, Manifest},
        AssetCollectionExt,
    },
    player_interaction::InteractionSystem,
    signals::{Emitter, SignalStrength, SignalType},
    simulation::{
        geometry::{Facing, MapGeometry, TilePos},
        SimulationSet,
    },
};
use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng};
use serde::{Deserialize, Serialize};

use self::{
    actions::CurrentAction,
    goals::Goal,
    impatience::ImpatiencePool,
    item_interaction::UnitInventory,
    unit_assets::UnitHandles,
    unit_manifest::{Unit, UnitData, UnitManifest},
};

use crate::organisms::OrganismBundle;

pub(crate) mod actions;
pub(crate) mod goals;
pub mod hunger;
pub(crate) mod impatience;
pub(crate) mod item_interaction;
mod reproduction;
pub(crate) mod unit_assets;
pub mod unit_manifest;

/// Controls the distribution of wandering durations on a per-unit-type basis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WanderingBehavior {
    /// How many actions will units take while wandering before picking a new goal?
    wander_durations: Vec<u16>,
    /// The relative probability of each value in `mean_free_wander_period`.
    weights: Vec<f32>,
}

impl WanderingBehavior {
    /// Randomly choose the number of actions to take while wandering.
    fn sample(&self, rng: &mut ThreadRng) -> u16 {
        // We can't store a WeightedIndex directly because it's not serializable.
        let weighted_index = WeightedIndex::new(&self.weights).unwrap();

        self.wander_durations[weighted_index.sample(rng)]
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
            weights,
        }
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
    /// What signals is this unit emitting?
    emitter: Emitter,
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
            emitter: Emitter {
                signals: vec![(SignalType::Unit(unit_id), SignalStrength::new(1.))],
            },
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
        app.init_resource::<UnitManifest>()
            .add_asset_collection::<UnitHandles>()
            .add_systems(
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
