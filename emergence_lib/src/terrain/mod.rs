use crate::tiles::IntoTileBundle;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapId, TilemapTileSize};
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ecs_tilemap::tiles::{TilePos, TileTexture};
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

pub mod generation;

pub static TERRAIN_TILE_IMAP: Lazy<IndexMap<TerrainType, &'static str>> = Lazy::new(|| {
    indexmap! {
        TerrainType::High => "tile-high.png",
        TerrainType::Impassable => "tile-impassable.png",
        TerrainType::Plain => "tile-plain.png",
    }
});

/// The tile size (hex tile width by hex tile height) in pixels of tile image assets.
pub const TERRAIN_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
/// The z-coordinate at which tiles are drawn.
pub const TERRAIN_TILEMAP_Z: f32 = 0.0;

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

impl IntoTileBundle for TerrainType {
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

/// Marker component for entities that are part of the terrain's tilemap
#[derive(Component)]
pub struct TerrainTilemap;

/// A [`WorldQuery`] specifying a search for `TileStorage` associated with a
/// `Tilemap` that has the `TerrainTilemap` component type.
#[derive(WorldQuery)]
pub struct TerrainStorage<'a> {
    pub storage: &'a TileStorage,
    _terrain_tile_map: With<TerrainTilemap>,
}
