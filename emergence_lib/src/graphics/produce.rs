//! The [`ProduceTilemap`] manages visualization of produce.
use crate as emergence_lib;
use crate::graphics::sprites::SpriteIndex;

use bevy::ecs::component::Component;

use emergence_macros::IterableEnum;
use std::path::PathBuf;

/// Enumerates produce sprites.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum ProduceSprite {
    /// Sprite representing food
    Food,
}

impl SpriteIndex for ProduceSprite {
    const ROOT_FOLDER: &'static str = "produce";

    fn leaf_path(&self) -> PathBuf {
        match self {
            ProduceSprite::Food => "tile-food-balls.png".into(),
        }
    }
}

/// Marker component for the produce tilemap
#[derive(Component, Clone, Copy, Debug)]
pub struct ProduceTilemap;
