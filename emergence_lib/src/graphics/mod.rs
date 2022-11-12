//! Utilities for defining and visualizing game graphics.

use crate::graphics::terrain::TerrainTilemap;
use crate::terrain::MapGeometry;
use bevy::app::{App, Plugin, StartupStage};
use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::log::info;
use bevy::prelude::Res;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::{
    HexCoordSystem, TilemapGridSize, TilemapId, TilemapTexture, TilemapType,
};
use bevy_ecs_tilemap::prelude::get_tilemap_center_transform;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;

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
        Tiles::terrain_tile_paths()
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

/// The grid size (hex tile width by hex tile height) in pixels.
///
/// Grid size should be the same for all tilemaps, as we want them to be congruent.
pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 48.0, y: 54.0 };

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

/// Enumerates the various tiles that exist in the game
///
/// These should be organized in alphabetical order.
pub enum Tiles {
    // Organism tiles
    /// Tile for ants
    OrganismAnt,
    /// Tile for plants
    OrganismPlant,
    /// Tile for fungi
    OrganismFungus,
    // Terrain tiles
    /// Tile for high terrain
    TerrainHigh,
    /// Tile for plains
    TerrainPlain,
    /// Tile for impassable terrain
    TerrainImpassable,
}

impl Tiles {
    /// Returns the path of the texture associated with each tile.
    ///
    /// These are assumed to live in the `emergence_game` assets folder. Therefore, they should be
    /// provided relative to that folder.
    pub fn tile_texture_path(&self) -> &'static str {
        use Tiles::*;
        match self {
            OrganismAnt => "tile-ant.png",
            OrganismFungus => "tile-fungus.png",
            OrganismPlant => "tile-plant.png",
            TerrainHigh => "tile-high.png",
            TerrainPlain => "tile-plain.png",
            TerrainImpassable => "tile-impassable.png",
        }
    }

    pub fn organism_tile_paths() -> Vec<&'static str> {
        use Tiles::*;
        [OrganismAnt, OrganismFungus, OrganismPlant]
            .into_iter()
            .map(|t| t.tile_texture_path())
            .collect()
    }

    pub fn terrain_tile_paths() -> Vec<&'static str> {
        use Tiles::*;
        [TerrainHigh, TerrainPlain, TerrainImpassable]
            .into_iter()
            .map(|t| t.tile_texture_path())
            .collect()
    }

    pub fn tile_texture_index(&self) -> TileTextureIndex {
        use Tiles::*;
        match self {
            // Organism tiles
            OrganismAnt => TileTextureIndex(0),
            OrganismPlant => TileTextureIndex(1),
            OrganismFungus => TileTextureIndex(2),
            // Terrain tiles
            TerrainPlain => TileTextureIndex(0),
            TerrainHigh => TileTextureIndex(1),
            TerrainImpassable => TileTextureIndex(2),
        }
    }
}

/// Manages the mapping between a tile and the layer it exists in, and its index within that layer.
pub struct TileRegistrar {
    map: HashMap<Tiles, (Layer, TileTextureIndex)>,
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
