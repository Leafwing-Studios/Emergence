//! Computes and displays helpful information about the state of the world.
//!
//! UI elements generated for / by this work belong in the `ui` module instead.

use bevy::prelude::*;

use crate::asset_management::manifest::{Id, Unit};

/// Systems and reources for communicating the state of the world to the player.
pub struct InfoVisPlugin;

impl Plugin for InfoVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(census);
    }
}

fn census(query: Query<(), With<Id<Unit>>>) {
    let n = query.iter().len();
    info!(n)
}
