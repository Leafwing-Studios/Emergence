//! Life stages are connected by life paths, creating a life cycle for each strain of organism.

use bevy::{
    prelude::Resource,
    utils::{HashMap, HashSet},
};

use super::Species;

/// A map of the [`LifeStages`](Species::LifeStage) for an organism of type `S`, connected by [`LifePath`]s.
///
/// This type is the high-level map for the entire [`Species`], and is stored as a resource.
///
/// This forms a finite state machine.
/// Paths may diverge, loop back onto themselves, terminate and so on.
#[derive(Resource, Default)]
pub struct LifeCycle<S: Species> {
    /// The set of all possible life stages for an organism of type `O`.
    pub life_stages: HashSet<S::LifeStage>,
    /// A map from the current life stage to the potential ways it could transform.
    pub life_paths: HashMap<S::LifeStage, LifePath>,
}

/// Paths that connect different life stages.
///
/// These are triggered when certain conditions are met for each organism,
/// causing them to change form and function.
// FIXME: placeholder type
pub struct LifePath;
