//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::{TerrainSprite, TerrainTilemap};
use crate::simulation::generation::GRID_SIZE;
use crate::terrain::MapGeometry;

use bevy::app::{App, Plugin, StartupStage};
use bevy::asset::AssetPath;
use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::log::info;
use bevy::prelude::{Res, ResMut};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapId, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;

use crate::graphics::debug::generate_debug_labels;
use crate::graphics::organisms::{OrganismSprite, OrganismTilemap};
use bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform;
use std::path::PathBuf;

pub mod debug;
pub mod organisms;
pub mod position;
pub mod terrain;

/// All of the code needed to draw things on screen.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .init_resource::<LayerRegister>()
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_terrain_layer)
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_organisms_layer)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_debug_labels);
    }
}

fn initialize_terrain_layer(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
    mut layer_register: ResMut<LayerRegister>,
) {
    let texture = TilemapTexture::Vector(
        TerrainSprite::all_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn().id();
    layer_register
        .map
        .insert(Layer::Terrain, TilemapId(tilemap_entity));
    let tile_storage = TileStorage::empty(map_geometry.size());

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

fn initialize_organisms_layer(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
    mut layer_register: ResMut<LayerRegister>,
) {
    let texture = TilemapTexture::Vector(
        OrganismSprite::all_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn().id();
    layer_register
        .map
        .insert(Layer::Organisms, TilemapId(tilemap_entity));
    let tile_storage = TileStorage::empty(map_geometry.size());

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: GRID_SIZE,
            map_type: MAP_TYPE,
            size: map_geometry.size(),
            storage: tile_storage,
            texture,
            tile_size: OrganismTilemap::TILE_SIZE,
            transform: get_tilemap_center_transform(
                &map_geometry.size(),
                &GRID_SIZE,
                &MAP_TYPE,
                OrganismTilemap::MAP_Z,
            ),
            ..Default::default()
        })
        .insert(OrganismTilemap);
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
    pub map: HashMap<Layer, TilemapId>,
}
pub trait IntoSprite: IterableEnum {
    /// Path to the folder containing texture assets for this particular group of entities.
    const ROOT_PATH: &'static str;
    /// Layer (tilemap) that this group of entities belongs to.
    const LAYER: Layer;

    /// Path of a particular entity variant.
    fn leaf_path(&self) -> &'static str;

    /// Returns ROOT_PATH + leaf_path().
    fn full_path(&self) -> AssetPath<'static> {
        let path = PathBuf::from(Self::ROOT_PATH).join(self.leaf_path());

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

    fn tile_bundle(&self, position: TilePos, layer_register: &Res<LayerRegister>) -> TileBundle {
        TileBundle {
            position,
            texture_index: self.tile_texture_index(),
            tilemap_id: *layer_register
                .map
                .get(&Self::LAYER)
                .expect("Layer not registered"),
            ..Default::default()
        }
    }
}
