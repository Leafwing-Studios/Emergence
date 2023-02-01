//! Organisms that cannot move.
//!
//! These are a special subset of structures which act on their own, go through life stages and must produce in order to survive.

use bevy::prelude::Bundle;

use crate::{
    items::recipe::RecipeId,
    simulation::geometry::TilePos,
    structures::{crafting::CraftingBundle, StructureBundle},
};

use super::{OrganismBundle, Species};

pub mod fungi;
pub mod plants;

/// Sessile organisms cannot move, and automatically process nutrients from their environment
#[derive(Bundle)]
pub struct SessileBundle<S: Species> {
    /// Sessile organisms are organisms
    pub organism_bundle: OrganismBundle<S>,

    /// Sessile organisms are structures
    pub structure_bundle: StructureBundle,

    /// Sessile organisms can craft things
    pub crafting_bundle: CraftingBundle,

    /// Which tile is this sessile organism on top of
    pub tile_pos: TilePos,
}

impl<S: Species> SessileBundle<S> {
    /// Create a new [`SessileBundle`] at the given `tile_pos`, without an active crafting recipe.
    pub fn new(tile_pos: TilePos) -> SessileBundle<S> {
        SessileBundle {
            organism_bundle: OrganismBundle::default(),
            structure_bundle: StructureBundle::default(),
            crafting_bundle: CraftingBundle::new(),
            tile_pos,
        }
    }

    /// Create a new [`SessileBundle`] at the given `tile_pos`, which will attempt to produce the provided `recipe` automatically.
    pub fn new_with_recipe(tile_pos: TilePos, recipe_id: RecipeId) -> SessileBundle<S> {
        SessileBundle {
            organism_bundle: OrganismBundle::default(),
            structure_bundle: StructureBundle::default(),
            crafting_bundle: CraftingBundle::new_with_recipe(recipe_id),
            tile_pos,
        }
    }
}
