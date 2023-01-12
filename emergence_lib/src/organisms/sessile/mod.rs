//! Organisms that cannot move.
//!
//! These are a special subset of structures which act on their own, go through life stages and must produce in order to survive.

use bevy::prelude::Bundle;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    items::Recipe,
    structures::{crafting::CraftingBundle, StructureBundle},
};

pub mod fungi;
pub mod plants;

/// Sessile organisms cannot move, and automatically process nutrients from their environment
#[derive(Bundle)]
pub struct SessileBundle {
    /// Sessile organisms are structures
    pub structure_bundle: StructureBundle,

    /// Sessile organisms can craft things
    pub crafting_bundle: CraftingBundle,

    /// Which tile is this sessile organism on top of
    pub tile_pos: TilePos,
}

impl SessileBundle {
    /// Create a new [`SessileBundle`] at the given `tile_pos`, which will attempt to produce the provided `recipe` automatically.
    pub fn new(tile_pos: TilePos, recipe: Recipe) -> SessileBundle {
        SessileBundle {
            structure_bundle: StructureBundle::default(),
            crafting_bundle: CraftingBundle::new(recipe),
            tile_pos,
        }
    }
}
