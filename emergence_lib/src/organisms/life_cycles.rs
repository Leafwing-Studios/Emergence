//! Life stages are connected by life paths, creating a life cycle for each strain of organism.

use bevy::{prelude::Resource, utils::HashMap};

use super::Species;

/// A map of the [`LifeStages`](Species::LifeStage) for an organism of type `S`, connected by [`LifePath`]s.
///
/// This type is the high-level map for the entire [`Species`], and is stored as a resource.
///
/// This forms a finite state machine.
/// Paths may diverge, loop back onto themselves, terminate and so on.
#[derive(Resource, Default)]
pub struct LifeCycle<S: Species> {
    /// Describes how a life stage can transition to other life stages.
    ///
    /// This is a map of the outgoing paths.
    pub life_paths: HashMap<S::LifeStage, LifePath<S>>,
}

/// Paths that connect different life stages.
///
/// These are triggered when certain conditions are met for each organism,
/// causing them to change form and function.
pub struct LifePath<S: Species> {
    /// The life stage that the organism is transitioning to.
    pub target: S::LifeStage,
    /// The conditions that must be met for the transition to occur.
    pub requirements: Requirements,
}

/// The condition that must be met for an organism to transition along a life path.
// FIXME: placeholder type
pub struct Requirements;
