//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::prelude::*;

use crate::simulation::geometry::Facing;

use self::crafting::CraftingPlugin;

pub mod crafting;

/// The data needed to build a structure
#[derive(Bundle, Default)]
pub struct StructureBundle {
    /// Data characterizing structures
    structure: Structure,
    /// The direction this structure is facing
    facing: Facing,
}

/// Structures are static buildings that take up one or more tile
#[derive(Default, Component, Clone)]
pub struct Structure;

/// The systems that make structures tick.
pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CraftingPlugin);
    }
}
