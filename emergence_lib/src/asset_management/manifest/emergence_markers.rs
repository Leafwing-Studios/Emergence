//! Code for Emergence-specific marker types

use crate::{
    items::{recipe::RecipeData, ItemData},
    units::UnitData,
};

use super::{structure::StructureData, terrain::TerrainData, Manifest};

/// The marker type for [`Id<Recipe>`](super::Id).
pub struct Recipe;

/// Stores the read-only definitions for all recipes.
pub(crate) type RecipeManifest = Manifest<Recipe, RecipeData>;

/// The marker type for [`Id<Unit>`](super::Id).
pub struct Unit;
/// Stores the read-only definitions for all units.
pub(crate) type UnitManifest = Manifest<Unit, UnitData>;

/// The marker type for [`Id<Structure>`](super::Id).
pub struct Structure;
/// Stores the read-only definitions for all structures.
pub(crate) type StructureManifest = Manifest<Structure, StructureData>;

/// The marker type for [`Id<Terrain>`](super::Id).
pub struct Terrain;
/// Stores the read-only definitions for all items.
pub(crate) type TerrainManifest = Manifest<Terrain, TerrainData>;

/// The marker type for [`Id<Item>`](super::Id).
pub struct Item;
/// Stores the read-only definitions for all items.
pub(crate) type ItemManifest = Manifest<Item, ItemData>;
