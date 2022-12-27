//! The [`OrganismsTilemap`] manages visualization of organisms.

use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;

use bevy::prelude::Component;

use emergence_macros::IterableEnum;
use std::path::PathBuf;

/// Enumerates organism sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum OrganismSprite {
    /// Sprite for an Ant
    Ant,
    /// Sprite for a Plant
    Plant,
    /// Sprite for fungi
    Fungi,
}

impl SpriteIndex for OrganismSprite {
    const ROOT_FOLDER: &'static str = "organisms";

    fn leaf_path(&self) -> PathBuf {
        match self {
            OrganismSprite::Ant => "tile-ant.png".into(),
            OrganismSprite::Fungi => "tile-fungus.png".into(),
            OrganismSprite::Plant => "tile-plant.png".into(),
        }
    }
}

/// Marker component for the organism tilemap.
#[derive(Component, Clone, Copy, Debug)]
pub struct OrganismsTilemap;
