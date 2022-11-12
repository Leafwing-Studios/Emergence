//! Generating and representing terrain as game objects.
use crate as emergence_lib;
use crate::enum_iter::IterableEnum;
use crate::graphics::{IntoSprite, LayerRegister};
use bevy::prelude::{Commands, Component, Entity, Res};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::tiles::TilePos;
use emergence_macros::IterableEnum;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;

/// The number of graphics from the center of the map to the edge
pub const MAP_RADIUS: u32 = 10;

/// Resource that stores information regarding the size of the game map.
pub struct MapGeometry {
    /// The radius, in graphics, of the map
    pub radius: u32,
    /// The location of the central tile
    pub center: TilePos,
    /// The [`TilemapSize`] of the map
    pub size: TilemapSize,
}

impl MapGeometry {
    /// Computes the total diameter from end-to-end of the game world
    #[inline]
    pub const fn diameter(&self) -> u32 {
        2 * self.radius + 1
    }

    /// Computes the [`TilemapSize`] of the game world
    #[inline]
    pub const fn size(&self) -> TilemapSize {
        self.size
    }

    /// Computes the [`TilePos`] of the tile at the center of this map.
    ///
    /// This is not (0,0) as `bevy_ecs_tilemap` works with `u32` coordinates.
    #[inline]
    pub const fn center(&self) -> TilePos {
        self.center
    }
}

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
#[derive(Clone, Copy, Hash, Eq, PartialEq, IterableEnum)]
pub enum TerrainType {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is impassable.
    Impassable,
    /// Terrain that has higher altitude compared to others.
    High,
}

impl TerrainType {
    /// Creates a tile enttiy corresponding to `self`'s [`TerrainType`] variant
    pub fn create_entity(
        &self,
        commands: &mut Commands,
        position: TilePos,
        layer_register: &Res<LayerRegister>,
    ) -> Entity {
        let mut builder = commands.spawn();

        builder.insert_bundle(self.tile_bundle(position, layer_register));
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
        TerrainType::variants()
            .collect::<Vec<TerrainType>>()
            .choose_weighted(rng, |terrain_type| {
                weights.get(terrain_type).copied().unwrap_or_default()
            })
            .copied()
    }
}
