//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::TerrainTilemap;
use crate::simulation::generation::GRID_SIZE;
use crate::terrain::{terrain_type::TerrainType, MapGeometry};

use bevy::app::{App, Plugin, StartupStage};
use bevy::asset::AssetPath;
use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::log::info;
use bevy::prelude::Res;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapId, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::prelude::get_tilemap_center_transform;
use bevy_ecs_tilemap::tiles::{TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;

use std::path::PathBuf;

pub mod organisms;
pub mod position;
pub mod terrain;

/// All of the code needed to draw things on screen.
///
/// All the startup systems of this stage run in in [`StartupStage::PreStartup`].
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .init_resource::<LayerRegister>()
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_terrain_layer);
    }
}

fn initialize_terrain_layer(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
) {
    let texture = TilemapTexture::Vector(
        TerrainType::all_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(map_geometry.size());

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: GRID_SIZE,
            map_type: MAP_TYPE,
            size: map_geometry.size(),
            storage: tile_storage,
            texture,
            tile_size: TerrainTilemap::TILE_SIZE,
            transform: get_tilemap_center_transform(
                &map_geometry.size(),
                &GRID_SIZE,
                &MAP_TYPE,
                TerrainTilemap::MAP_Z,
            ),
            ..Default::default()
        })
        .insert(TerrainTilemap);
}

/// We use a hexagonal map with "pointy-topped" (row oriented) graphics, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// We are using a map with hexagonal graphics.
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// Enumerates the different layers we are organizing our graphics into
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// Organisms layer
    Organisms,
    /// Terrain layer
    Terrain,
    /// Produce layer
    Produce,
}

/// Manages the mapping between layers and `bevy_ecs_tilemap` tilemaps
#[derive(Default)]
pub struct LayerRegister {
    /// A map from Layer to TilemapId
    map: HashMap<Layer, TilemapId>,
}
pub trait IntoSprite: IterableEnum + Into<u32> {
    /// Path to the folder containing texture assets for this particular group of entities.
    const ROOT_PATH: &'static str;

    /// Path of a particular entity variant.
    fn leaf_path(&self) -> &'static str;

    /// Returns ROOT_PATH + leaf_path().
    fn full_path(&self) -> AssetPath {
        let path = PathBuf::from(Self::ROOT_PATH);
        path.join(self.leaf_path());

        AssetPath::new(path, None)
    }

    fn all_paths() -> Vec<AssetPath<'static>> {
        Self::variants()
            .map(|variant| variant.full_path())
            .collect()
    }

    fn tile_texture_index(&self) -> TileTextureIndex {
        TileTextureIndex(self.index() as u32)
    }
}

/// Manages the mapping between a tile and the layer it exists in, and its index within that layer.
pub struct TileRegistrar {
    map: HashMap<TileSprite, (Layer, TileTextureIndex)>,
    index_allocator: TextureIndexAllocator,
}

/// Helper for producing tile texture indices for each layer.
pub struct TextureIndexAllocator {
    /// Keeps track of the next unused index for each layer.
    map: HashMap<Layer, u32>,
}

/// A type that can be transformed into a tile that is compatible with [`bevy_ecs_tilemap`].
pub trait IntoTileBundle {
    /// The corresponding [`TileTextureIndex`] and the [`TilemapId`] layer that it belongs to.
    fn tile_texture(
        &self,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
    ) -> (TilemapId, TileTextureIndex);

    /// The asset path to the [`TileTextureIndex`].
    fn tile_texture_path(&self) -> &'static str;

    /// Uses the data stored in `self` to create a new, matching [`TileBundle`].
    fn as_tile_bundle(
        &self,
        tilemap_id: TilemapId,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
        position: TilePos,
    ) -> TileBundle {
        TileBundle {
            position,
            tilemap_id,
            texture_index: self.tile_texture(tilemap_ids).1,
            ..Default::default()
        }
    }
}
