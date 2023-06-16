//! Controls random number generation.
//!
//! Storing the random number generator in a resource allows us to generate worlds deterministically.
// TODO: replace with bevy_turborand.

use bevy::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

/// A global source of entropy.
#[derive(Debug, Clone, Resource, PartialEq, Eq, Deref, DerefMut)]
pub(crate) struct GlobalRng(SmallRng);

impl GlobalRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(SmallRng::seed_from_u64(seed))
    }

    pub(crate) fn get_mut(&mut self) -> &mut SmallRng {
        &mut self.0
    }
}
