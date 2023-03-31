use bevy::reflect::{FromReflect, Reflect};

use crate::items::recipe::RecipeData;

use super::Manifest;

/// The marker type for [`Id<Recipe>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Recipe;

/// Stores the read-only definitions for all recipes.
pub(crate) type RecipeManifest = Manifest<Recipe, RecipeData>;
