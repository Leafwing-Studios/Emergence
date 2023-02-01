//! Units are organisms that can move freely.

use crate::curves::{BottomClampedLine, Mapping, Sigmoid};
use crate::simulation::map::TilePos;
use bevy::prelude::*;

use self::behavior::events::{
    DropOffThisTurn, IdleThisTurn, MoveThisTurn, PickUpThisTurn, WorkThisTurn,
};
use self::behavior::CurrentGoal;

mod act;
mod behavior;
mod pathfinding;

/// Available types of units
pub enum UnitType {
    /// A worker ant
    Ant,
}

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
}

/// Data characterizing ants
#[derive(Component, Clone, Default)]
pub struct Ant;

/// A worker ant
#[derive(Bundle)]
pub struct AntBundle {
    /// Data characterizing ants
    ant: Ant,
    /// Ants are units.
    unit_bundle: UnitBundle,
    /// Position in the world
    position: TilePos,
}

impl AntBundle {
    /// Creates a new [`AntBundle`]
    pub fn new(position: TilePos) -> Self {
        Self {
            ant: Ant,
            unit_bundle: UnitBundle {
                ..Default::default()
            },
            position,
        }
    }
}

/// System labels for unit behavior
#[derive(SystemLabel)]
pub enum UnitSystem {
    /// Pick a higher level goal to pursue
    ChooseGoal,
    /// Pick an action that will get the agent closer to the goal being pursued
    ChooseAction,
    /// Carry out the chosen action
    Act,
}

/// Contains unit behavior
pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnitTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .insert_resource(SignalTransducer::<BottomClampedLine>::default())
            .add_event::<IdleThisTurn>()
            .add_event::<MoveThisTurn>()
            .add_event::<PickUpThisTurn>()
            .add_event::<DropOffThisTurn>()
            .add_event::<WorkThisTurn>()
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
#[derive(Resource, Debug)]
struct UnitTimer(Timer);

/// Transforms a signal into a weight used to make decisions.
///
/// The transduction is modelled by mapping the signal to a weight using a curve.
#[derive(Resource)]
pub struct SignalTransducer<C: Mapping> {
    /// Curve used to model transduction.
    curve: C,
}

impl SignalTransducer<Sigmoid> {
    /// Creates a [`Sigmoid`]-based transducer.
    pub fn new(
        min: f32,
        max: f32,
        first_percentile: f32,
        last_percentile: f32,
    ) -> SignalTransducer<Sigmoid> {
        SignalTransducer {
            curve: Sigmoid::new(min, max, first_percentile, last_percentile),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for SignalTransducer<Sigmoid> {
    fn default() -> Self {
        SignalTransducer {
            curve: Sigmoid::new(0.0, 0.1, 0.01, 0.09),
        }
    }
}

impl SignalTransducer<BottomClampedLine> {
    /// Creates a [`BottomClampedLine`]-based transducer.
    pub fn new(p0: Vec2, p1: Vec2) -> SignalTransducer<BottomClampedLine> {
        SignalTransducer {
            curve: BottomClampedLine::new_from_points(p0, p1),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for SignalTransducer<BottomClampedLine> {
    fn default() -> Self {
        SignalTransducer {
            curve: BottomClampedLine::new_from_points(Vec2::new(0.0, 0.0), Vec2::new(0.01, 1.0)),
        }
    }
}
