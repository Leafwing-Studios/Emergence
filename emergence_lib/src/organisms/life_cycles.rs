//! Life stages are connected by life paths, creating a life cycle for each strain of organism.

use bevy::{
    prelude::Component,
    utils::{HashMap, HashSet},
};

use super::Species;

/// A collection of [`LifeStages`](Species::LifeStage) for an organism of type `S`, connected by [`LifePath`]s
#[derive(Component, Default)]
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
