//! Sessile organisms are both structures and orgnanisms.
//!
//! These are typically plants and fungi.

use bevy::prelude::*;

use crate::{items::recipe::RecipeId, organisms::OrganismBundle, simulation::geometry::TilePos};

use super::{crafting::CraftingBundle, StructureBundle, StructureId};

/// Sessile organisms cannot move, and automatically process nutrients from their environment
#[derive(Bundle)]
struct SessileBundle {
    /// Sessile organisms are organisms
    organism_bundle: OrganismBundle,

    /// Sessile organisms are structures
    structure_bundle: StructureBundle,

    /// Sessile organisms can craft things
    crafting_bundle: CraftingBundle,
}

impl SessileBundle {
    /// Create a new [`SessileBundle`] at the given `tile_pos`, without an active crafting recipe.
    pub fn new(tile_pos: TilePos, structure_id: StructureId) -> SessileBundle {
        SessileBundle {
            organism_bundle: OrganismBundle::default(),
            structure_bundle: StructureBundle::new(structure_id, tile_pos),
            crafting_bundle: CraftingBundle::new(),
        }
    }

    /// Create a new [`SessileBundle`] at the given `tile_pos`, which will attempt to produce the provided `recipe` automatically.
    pub fn new_with_recipe(
        tile_pos: TilePos,
        recipe_id: RecipeId,
        structure_id: StructureId,
    ) -> SessileBundle {
        SessileBundle {
            organism_bundle: OrganismBundle::default(),
            structure_bundle: StructureBundle::new(structure_id, tile_pos),
            crafting_bundle: CraftingBundle::new_with_recipe(recipe_id),
        }
    }
}
