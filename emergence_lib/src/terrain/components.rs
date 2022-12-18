//! Data for characterizing entities as terrain
use crate::enum_iter::IterableEnum;
use crate::graphics::sprites::IntoSprite;
use crate::graphics::terrain::TerrainSprite;
use crate::graphics::Tilemap;
use bevy::ecs::component::Component;

/// Component representing plain terrain
#[derive(Component, Clone, Copy)]
pub struct PlainTerrain;

impl IntoSprite for PlainTerrain {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Terrain
    }

    fn index(&self) -> u32 {
        TerrainSprite::Plain.index() as u32
    }
}

/// Component representing impassable terrain.
#[derive(Component, Clone, Copy)]
pub struct ImpassableTerrain;

impl IntoSprite for ImpassableTerrain {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Terrain
    }

    fn index(&self) -> u32 {
        TerrainSprite::Impassable.index() as u32
    }
}

/// The marker component for high terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct HighTerrain;

impl IntoSprite for HighTerrain {
    fn tilemap(&self) -> Tilemap {
        Tilemap::Terrain
    }

    fn index(&self) -> u32 {
        TerrainSprite::High.index() as u32
    }
}
