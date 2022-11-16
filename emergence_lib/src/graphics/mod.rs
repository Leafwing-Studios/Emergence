//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::TerrainTilemap;
use crate::simulation::generation::GRID_SIZE;
use crate::terrain::{MapGeometry, TerrainType};

use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::asset::AssetPath;
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::{Res, ResMut, Resource};
use bevy::log::info;
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapId, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;

use crate::graphics::debug::generate_debug_labels;
use crate::graphics::organisms::{OrganismSprite, OrganismTilemap};
use bevy::prelude::Added;
use bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform;
use emergence_macros::IterableEnum;
use std::path::PathBuf;

use crate as emergence_lib;

pub mod debug;
pub mod organisms;
pub mod position;
pub mod terrain;

/// All of the code needed to draw things on screen.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .init_resource::<TilemapRegister>()
            .init_resource::<MapGeometry>()
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_terrain_layer)
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_organisms_layer)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_debug_labels)
            .add_startup_system_to_stage(CoreStage::First, populate_graphical_layers);
    }
}

/// Initializes the terrain graphical layer (tilemap).
fn initialize_terrain_layer(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
    mut layer_register: ResMut<TilemapRegister>,
) {
    let texture = TilemapTexture::Vector(
        TerrainType::all_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn_empty().id();
    layer_register
        .register
        .insert(Layer::Terrain.index(), TilemapId(tilemap_entity));
    let tile_storage = TileStorage::empty(map_geometry.size());

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
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

/// Initializes the organisms graphical layer (tilemap).
fn initialize_organisms_layer(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
    mut layer_register: ResMut<TilemapRegister>,
) {
    let texture = TilemapTexture::Vector(
        OrganismSprite::all_paths()
            .into_iter()
            .map(|p| asset_server.load(p))
            .collect(),
    );

    let tilemap_entity = commands.spawn_empty().id();
    layer_register
        .register
        .insert(Layer::Organisms.index(), TilemapId(tilemap_entity));
    let tile_storage = TileStorage::empty(map_geometry.size());

    info!("Inserting TilemapBundle...");
    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
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

/// Populate graphical layers based on entities that have a newly added component which implements
/// [`IntoSprite`].
fn populate_graphical_layers(
    mut commands: Commands,
    terrain_query: Query<(Entity, &TerrainType, &TilePos), Added<TerrainType>>,
    organisms_query: Query<(Entity, &OrganismSprite, &TilePos), Added<OrganismSprite>>,
    tilemap_register: Res<TilemapRegister>,
) {
    for (entity, terrain, position) in terrain_query.iter() {
        commands
            .entity(entity)
            .insert(terrain.tile_bundle(*position, &tilemap_register));
    }

    for (entity, organism, position) in organisms_query.iter() {
        commands
            .entity(entity)
            .insert(organism.tile_bundle(*position, &tilemap_register));
    }
}

/// We use a hexagonal map with "pointy-topped" (row oriented) graphics, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// We are using a map with hexagonal graphics.
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// Enumerates the different layers we are organizing our graphics into
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, IterableEnum)]
pub enum Layer {
    /// Organisms layer
    Organisms,
    /// Terrain layer
    Terrain,
    /// Produce layer
    Produce,
}

/// Manages the mapping between [`Layer`]s and `bevy_ecs_tilemap` tilemaps
#[derive(Resource, Default, Debug)]
pub struct TilemapRegister {
    /// A vector consisting of the tilemaps initialized with each corresponding [`Layer`] variant.
    pub register: Vec<TilemapId>,
}

/// Defines how to map from variants of this type into a sprite asset that can be loaded into the game.
pub trait IntoSprite: IterableEnum {
    /// Path to the folder containing texture assets for this particular group of entities.
    const ROOT_PATH: &'static str;
    /// Layer (tilemap) that this group of entities belongs to.
    const LAYER: Layer;

    /// Path of a particular entity variant.
    fn leaf_path(&self) -> &'static str;

    /// Returns `ROOT_PATH + leaf_path()`.
    fn full_path(&self) -> AssetPath<'static> {
        let path = PathBuf::from(Self::ROOT_PATH).join(self.leaf_path());

        AssetPath::new(path, None)
    }

    /// Returns all the sprite paths in `ROOT_PATH`
    fn all_paths() -> Vec<AssetPath<'static>> {
        Self::variants()
            .map(|variant| variant.full_path())
            .collect()
    }

    /// Returns this item's index as a [`TileTextureIndex`].
    fn tile_texture_index(&self) -> TileTextureIndex {
        TileTextureIndex(self.index() as u32)
    }

    /// Creates a [`TileBundle`] for an entity of this type, which can be used to initialize it in [`bevy_ecs_tilemap`].
    fn tile_bundle(
        &self,
        position: TilePos,
        tilemap_register: &Res<TilemapRegister>,
    ) -> TileBundle {
        TileBundle {
            position,
            texture_index: self.tile_texture_index(),
            tilemap_id: *tilemap_register
                .register
                .get(Self::LAYER.index())
                .unwrap_or_else(|| panic!("Layer {:?} not registered", Self::LAYER)),
            ..Default::default()
        }
    }
}
