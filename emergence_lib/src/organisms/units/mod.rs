//! Units are organisms that can move freely.

use crate::curves::{BottomClampedLine, Mapping, Sigmoid};
use crate::organisms::OrganismBundle;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::TileBundle;
use bevy_ecs_tilemap::tiles::TilePos;

use self::behavior::CurrentGoal;

mod act;
mod behavior;
mod pathfinding;

/// Marker component for [`UnitBundle`]
#[derive(Component, Clone, Default)]
pub struct Unit;

/// An organism that can move around freely.
#[derive(Bundle, Default)]
pub struct UnitBundle {
    /// Marker component.
    unit: Unit,
    /// What is the unit trying to do
    current_task: CurrentGoal,
    /// A unit is an organism.
    #[bundle]
    organism_bundle: OrganismBundle,
}

/// Marker component for worker ants
#[derive(Component, Clone, Default)]
pub struct Ant;

/// A worker ant
#[derive(Bundle, Default)]
pub struct AntBundle {
    /// Marker struct.
    ant: Ant,
    /// Ants are units.
    #[bundle]
    unit_bundle: UnitBundle,
    /// Data needed to visualize the ant.
    #[bundle]
    tile_bundle: TileBundle,
}

impl AntBundle {
    /// Creates a new [`AntBundle`]
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            unit_bundle: UnitBundle {
                organism_bundle: OrganismBundle {
                    ..Default::default()
                },
                ..Default::default()
            },
            tile_bundle: todo!(),
            ..Default::default()
        }
    }
}

/// System labels for unit behavior
#[derive(SystemLabel)]
pub enum UnitSystem {
    ChooseGoal,
    ChooseAction,
    Act,
}

/// Contains unit behavior
pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnitTimer(Timer::from_seconds(0.5, true)))
            .insert_resource(PheromoneTransducer::<BottomClampedLine>::default())
            .add_system(behavior::choose_goal.label(UnitSystem::ChooseGoal))
            .add_system(
                behavior::choose_action
                    .label(UnitSystem::ChooseAction)
                    .after(UnitSystem::ChooseGoal),
            )
            .add_system(
                act::act
                    .label(UnitSystem::Act)
                    .after(UnitSystem::ChooseAction),
            );
    }
}
/// Global timer that controls when units should act
struct UnitTimer(Timer);

/// Transduces a pheromone signal into a weight used to make decisions.
///
/// The transduction is modelled by mapping the signal to a weight using a curve.
pub struct PheromoneTransducer<C: Mapping> {
    /// Curve used to model transduction.
    curve: C,
}

impl PheromoneTransducer<Sigmoid> {
    /// Creates a [`Sigmoid`]-based transducer.
    pub fn new(
        min: f32,
        max: f32,
        first_percentile: f32,
        last_percentile: f32,
    ) -> PheromoneTransducer<Sigmoid> {
        PheromoneTransducer {
            curve: Sigmoid::new(min, max, first_percentile, last_percentile),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for PheromoneTransducer<Sigmoid> {
    fn default() -> Self {
        PheromoneTransducer {
            curve: Sigmoid::new(0.0, 0.1, 0.01, 0.09),
        }
    }
}

impl PheromoneTransducer<BottomClampedLine> {
    /// Creates a [`BottomClampedLine`]-based transducer.
    pub fn new(p0: Vec2, p1: Vec2) -> PheromoneTransducer<BottomClampedLine> {
        PheromoneTransducer {
            curve: BottomClampedLine::new_from_points(p0, p1),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for PheromoneTransducer<BottomClampedLine> {
    fn default() -> Self {
        PheromoneTransducer {
            curve: BottomClampedLine::new_from_points(Vec2::new(0.0, 0.0), Vec2::new(0.01, 1.0)),
        }
    }
}
