//! Logic for buildings that move items around.

use bevy::prelude::*;

/// A building that spits out items.
#[derive(Component)]
pub(super) struct EmitsItems;

/// A building that takes in items.
#[derive(Component)]
pub(super) struct AbsorbsItems;
