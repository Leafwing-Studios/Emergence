//! Life stages are connected by life paths, creating a life cycle for each strain of organism.

use bevy::{
    prelude::Component,
    utils::{HashMap, HashSet},
};

use super::OrganismKind;

/// A collection of [`LifeStage`]s for an organism of type `O`, connected by [`LifePath`]s
#[derive(Component, Default)]
pub struct LifeCycle<O: OrganismKind> {
    /// The set of all possible life cycles for an organism of type `O`.
    pub life_stages: HashSet<O::LifeStage>,
    /// A map from the current life stage to the potential ways it could transform.
    pub life_paths: HashMap<O::LifeStage, LifePath>,
}

/// Paths that connect different life stages.
///
/// These are triggered when certain conditions are met for each organism,
/// causing them to change form and function.
pub struct LifePath;
