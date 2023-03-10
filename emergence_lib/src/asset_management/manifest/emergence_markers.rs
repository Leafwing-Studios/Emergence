//! Code for Emergence-specific marker types

use crate::{
    items::{recipe::RecipeData, ItemData},
    structures::StructureData,
    units::UnitData,
};

use super::Manifest;

/// The marker type for [`Id<Recipe>`](super::Id).
pub(crate) struct Recipe;

/// Stores the read-only definitions for all recipes.
pub(crate) type RecipeManifest = Manifest<Recipe, RecipeData>;

/// The marker type for [`Id<Unit>`](super::Id).
pub(crate) struct Unit;
/// Stores the read-only definitions for all units.
pub(crate) type UnitManifest = Manifest<Unit, UnitData>;

/// The marker type for [`Id<Structure>`](super::Id).
pub(crate) struct Structure;
/// Stores the read-only definitions for all structures.
pub(crate) type StructureManifest = Manifest<Structure, StructureData>;

/// The marker type for [`Id<Item>`](super::Id).
pub(crate) struct Item;
/// Stores the read-only definitions for all items.
pub(crate) type ItemManifest = Manifest<Item, ItemData>;
