//! Units are organisms that can move freely.

use crate::{
    asset_management::{
        manifest::{plugin::ManifestPlugin, Id, Manifest},
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
    age::Age,
    goals::Goal,
    impatience::ImpatiencePool,
    item_interaction::UnitInventory,
    unit_assets::UnitHandles,
    unit_manifest::{RawUnitManifest, Unit, UnitData},
};

use crate::organisms::OrganismBundle;

pub(crate) mod actions;
pub mod age;
pub(crate) mod goals;
pub mod hunger;
pub(crate) mod impatience;
pub(crate) mod item_interaction;
pub(crate) mod unit_assets;
pub mod unit_manifest;

/// Controls the distribution of wandering durations on a per-unit-type basis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WanderingBehavior {
    /// How many actions will units take while wandering before picking a new goal?
    ///
    /// The [`f32`] represents the relative probability of each value.
    wander_durations: Vec<(u16, f32)>,
}

impl WanderingBehavior {
    /// Randomly choose the number of actions to take while wandering.
    fn sample(&self, rng: &mut ThreadRng) -> u16 {
        let weights = self.wander_durations.iter().map(|(_, weight)| *weight);
        let dist = WeightedIndex::new(weights).unwrap();
        let index = dist.sample(rng);
        self.wander_durations[index].0
    }
}

impl FromIterator<(u16, f32)> for WanderingBehavior {
    fn from_iter<T: IntoIterator<Item = (u16, f32)>>(iter: T) -> Self {
        let wander_durations = Vec::from_iter(iter);

        WanderingBehavior { wander_durations }
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
    /// The current and max age of the unit.
    age: Age,
    /// Organism data
    organism_bundle: OrganismBundle,
    /// Makes units pickable
    raycast_mesh: RaycastMesh<Unit>,
    /// The mesh used for raycasting
    mesh: Handle<Mesh>,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl UnitBundle {
    /// Controls the strength of the unit's signal production.
    ///
    /// Increasing this value will make all units signal stronger,
    /// and increase the frequency at which units attempt to flee crowding.
    const UNIT_EMITTER_STRENGTH: f32 = 0.5;

    /// Initializes a new unit.
    ///
    /// It will be just born, and full.
    pub(crate) fn newborn(
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
                signals: vec![(
                    SignalType::Unit(unit_id),
                    SignalStrength::new(Self::UNIT_EMITTER_STRENGTH),
                )],
            },
            age: Age::newborn(unit_data.max_age),
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

    //// Generates a randomized unit.
    ///
    /// This is used for world generation.
    pub(crate) fn randomized(
        unit_id: Id<Unit>,
        tile_pos: TilePos,
        unit_data: UnitData,
        unit_handles: &UnitHandles,
        map_geometry: &MapGeometry,
        rng: &mut ThreadRng,
    ) -> Self {
        let scene_handle = unit_handles.scenes.get(&unit_id).unwrap();
        let mut energy_pool = unit_data.organism_variety.energy_pool;
        energy_pool.randomize(rng);
        let age = Age::randomized(rng, unit_data.max_age);

        UnitBundle {
            unit_id,
            tile_pos,
            facing: Facing::default(),
            current_goal: Goal::default(),
            impatience: ImpatiencePool::new(unit_data.max_impatience),
            current_action: CurrentAction::default(),
            held_item: UnitInventory::default(),
            emitter: Emitter {
                signals: vec![(
                    SignalType::Unit(unit_id),
                    SignalStrength::new(Self::UNIT_EMITTER_STRENGTH),
                )],
            },
            age,
            organism_bundle: OrganismBundle::new(energy_pool, unit_data.organism_variety.lifecycle),
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
        app.add_plugin(ManifestPlugin::<RawUnitManifest>::new())
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
                    hunger::check_for_hunger
                        // Avoid a delay
                        .before(UnitSystem::ChooseNewAction)
                        // Make sure to overwrite any existing goal
                        .after(UnitSystem::ChooseGoal),
                    age::aging,
                )
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
    }
}
