use crate::config::TERRAIN_TILE_IMAP;
use crate::tiles::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TilePos, TileTexture};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

/// The marker component for plain terrain.
#[derive(Component, Clone, Copy)]
pub struct PlainTerrain;

/// The marker component for impassable terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct ImpassableTerrain;

/// The marker component for high terrain.
#[derive(Component, Clone, Copy, Default)]
pub struct HighTerrain;

/// Available terrain types.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum TerrainType {
    Plain,
    Impassable,
    High,
}

impl IntoTile for TerrainType {
    /// The associated tile texture
    fn tile_texture(&self) -> TileTexture {
        TileTexture(TERRAIN_TILE_IMAP.get_index_of(self).unwrap() as u32)
    }

    /// The path to the associated tile texture
    fn tile_texture_path(&self) -> &'static str {
        TERRAIN_TILE_IMAP.get(self).unwrap()
    }
}

impl TerrainType {
    /// Creates a tile enttiy corresponding to `self`'s [`TerrainType`] variant
    pub fn create_entity(
        &self,
        commands: &mut Commands,
        tilemap_id: TilemapId,
        position: TilePos,
    ) -> Entity {
        let mut builder = commands.spawn();

        builder.insert_bundle(self.as_tile_bundle(tilemap_id, position));
        match self {
            TerrainType::Plain => {
                builder.insert(PlainTerrain);
            }
            TerrainType::Impassable => {
                builder.insert(ImpassableTerrain);
            }
            TerrainType::High => {
                builder.insert(HighTerrain);
            }
        }
        builder.id()
    }
}

impl Distribution<TerrainType> for Standard {
    /// Choose a [`TerrainType`] at weighted-random
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TerrainType {
        let c: f32 = rng.gen();
        if c < 0.1 {
            TerrainType::High
        } else if c < 0.2 {
            TerrainType::Impassable
        } else {
            TerrainType::Plain
        }
    }
}
