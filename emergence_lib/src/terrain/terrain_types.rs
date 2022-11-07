//! The different types of physical terrain.

use crate::tiles::IntoTileBundle;
use crate::tiles::LayerType;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
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
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is impassable.
    Impassable,
    /// Terrain that has higher altitude compared to others.
    High,
}

impl TerrainType {
    /// The set of all possible [`TerrainType`] variants
    const ALL_CHOICES: [TerrainType; 3] = [
        TerrainType::Plain,
        TerrainType::Impassable,
        TerrainType::High,
    ];
}

impl IntoTileBundle for TerrainType {
    /// The associated tile texture
    fn tile_texture(
        &self,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
    ) -> (TilemapId, TileTextureIndex) {
        todo!()
    }

    /// The path to the associated tile texture
    fn tile_texture_path(&self) -> &'static str {
        todo!()
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

        todo!();
        //builder.insert_bundle(self.as_tile_bundle(tilemap_id, position));
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

    /// Choose a random terrain tile based on the given weights
    pub fn choose_random<R: Rng + ?Sized>(
        rng: &mut R,
        weights: &HashMap<TerrainType, f32>,
    ) -> Result<TerrainType, WeightedError> {
        TerrainType::ALL_CHOICES
            .choose_weighted(rng, |terrain_type| {
                weights.get(terrain_type).copied().unwrap_or_default()
            })
            .copied()
    }
}
