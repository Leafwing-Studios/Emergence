//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::{TerrainSprite, TerrainTilemap};
use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::{Res, ResMut, Resource};
use bevy::log::info;
use bevy::prelude::{StageLabel, SystemStage};
use bevy_ecs_tilemap::map::{
    HexCoordSystem, TilemapGridSize, TilemapId, TilemapTexture, TilemapTileSize, TilemapType,
};
use bevy_ecs_tilemap::prelude::get_tilemap_center_transform;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ecs_tilemap::TilemapBundle;

use crate as emergence_lib;
use crate::graphics::organisms::{OrganismSprite, OrganismsTilemap};
#[cfg(feature = "debug_tools")]
use debug_tools::debug_ui::*;
use emergence_macros::IterableEnum;

use crate::graphics::produce::{ProduceSprite, ProduceTilemap};
use crate::graphics::sprites::{IntoSprite, SpriteIndex};
use crate::organisms::structures::{Fungi, Plant};
use crate::organisms::units::Ant;
use crate::simulation::map::MapGeometry;
use crate::terrain::components::{HighTerrain, PlainTerrain, RockyTerrain};
use bevy_trait_query::{ChangedOne, RegisterExt};

pub mod organisms;
pub mod produce;
pub mod sprites;
pub mod terrain;
pub mod ui;

/// All of the code needed to draw things on screen.
pub struct GraphicsPlugin;

/// The stages in which graphics systems run
#[derive(StageLabel)]
pub enum GraphicsStage {
    /// Stage in which tilemap initialization happens, should run after [`StartupStage::Startup`]
    TilemapInitialization,
    /// Stage in which debug label generation happens, should run after [`TilemapInitialization`](GraphicsStage::TilemapInitialization)
    DebugLabelGeneration,
}

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .register_component_as::<dyn IntoSprite, Ant>()
            .register_component_as::<dyn IntoSprite, Fungi>()
            .register_component_as::<dyn IntoSprite, Plant>()
            .register_component_as::<dyn IntoSprite, HighTerrain>()
            .register_component_as::<dyn IntoSprite, RockyTerrain>()
            .register_component_as::<dyn IntoSprite, PlainTerrain>()
            .init_resource::<TilemapRegister>()
            .add_startup_stage_after(
                StartupStage::Startup,
                GraphicsStage::TilemapInitialization,
                SystemStage::parallel(),
            )
            .add_startup_stage_after(
                GraphicsStage::TilemapInitialization,
                GraphicsStage::DebugLabelGeneration,
                SystemStage::parallel(),
            )
            // we put these systems in PostStartup, because we need the MapGeometry resource ready
            .add_startup_system_to_stage(GraphicsStage::TilemapInitialization, initialize_tilemaps)
            .add_system_to_stage(CoreStage::PreUpdate, update_sprites);

        #[cfg(feature = "debug_tools")]
        app.add_startup_system_to_stage(GraphicsStage::DebugLabelGeneration, initialize_infotext)
            .add_system_to_stage(CoreStage::Update, change_infotext);
    }
}

/// Initialize required tilemaps.
fn initialize_tilemaps(
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    asset_server: Res<AssetServer>,
    mut layer_register: ResMut<TilemapRegister>,
) {
    Tilemap::variants().for_each(|tilemap| {
        let entity = tilemap.spawn(&mut commands, &map_geometry, &asset_server);
        layer_register
            .register
            .insert(tilemap.index(), TilemapId(entity));
    });
}

/// Update entities that have a newly added/changed component which implements [`IntoSprite`] with
/// new `bevy_ecs_tilemap` [`TileBundle`](bevy_ecs_tilemap::tiles::TileBundle) information.
fn update_sprites(
    mut commands: Commands,
    into_sprites_query: Query<(Entity, &TilePos, ChangedOne<&dyn IntoSprite>)>,
    tilemap_register: Res<TilemapRegister>,
) {
    into_sprites_query.for_each(|(entity, position, maybe_sprite)| {
        if let Some(sprite) = maybe_sprite {
            commands
                .entity(entity)
                .insert(sprite.tile_bundle(*position, &tilemap_register));
        }
    });
}

/// We use a hexagonal map with "pointy-topped" (row oriented) tiles, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// We are using a map with hexagonal tiles.
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// Enumerates the different tilemaps we are organizing our graphics into
///
/// These should be ordered by their z-height. So, the terrain tilemap is the lowest (z-height `0`),
/// the organism tilemap is above it, etc.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, IterableEnum)]
pub enum Tilemap {
    /// Terrain tilemap
    Terrain,
    /// Organisms tilemap
    Organisms,
    /// Produce tilemap
    Produce,
}

impl Tilemap {
    /// The tile size (hex tile width by hex tile height) in pixels of the tilemap's tile image assets.
    ///
    /// Note that in general, a regular hexagon "pointy top" (row oriented) hexagon has a
    /// `width:height` ratio of [`sqrt(3.0)/2.0`](https://www.redblobgames.com/grids/hexagons/#basics),
    /// but because pixels are integer values, there will usually be some rounding, so tiles will
    /// only approximately match this ratio.
    pub const fn tile_size(&self) -> TilemapTileSize {
        // Currently all tilemaps have the same tile size
        TilemapTileSize { x: 48.0, y: 54.0 }
    }

    /// The grid size of this tilemap, in pixels.
    ///
    /// This can differ from [`tile_size`](Self::tile_size), in order to overlap tiles, or pad tiles.
    pub fn grid_size(&self) -> TilemapGridSize {
        // Currently all tilemaps have the same grid size as their tile size
        self.tile_size().into()
    }

    /// The z-height of this tile map, it is the same as the index of the tilemap's variant
    /// in the [`Tilemap`] enum
    pub const fn z_height(&self) -> f32 {
        *self as usize as f32
    }

    /// Loads the texture for this tilemap, using its corresponding [`SpriteIndex`] implementor
    pub fn load_texture(&self, asset_server: &AssetServer) -> TilemapTexture {
        match self {
            Tilemap::Terrain => TerrainSprite::load(asset_server),
            Tilemap::Organisms => OrganismSprite::load(asset_server),
            Tilemap::Produce => ProduceSprite::load(asset_server),
        }
    }

    /// Spawns tilemap component associated with each variant
    pub fn spawn(
        &self,
        commands: &mut Commands,
        map_geometry: &MapGeometry,
        asset_server: &AssetServer,
    ) -> Entity {
        info!("Inserting TilemapBundle for {:?}...", self);

        let texture = self.load_texture(asset_server);
        let tile_size = self.tile_size();
        let grid_size = self.grid_size();

        let mut entity_commands = commands.spawn(TilemapBundle {
            grid_size,
            map_type: MAP_TYPE,
            size: map_geometry.size(),
            texture,
            tile_size,
            transform: get_tilemap_center_transform(
                &map_geometry.size(),
                &grid_size,
                &MAP_TYPE,
                self.z_height(),
            ),
            ..Default::default()
        });

        match self {
            Tilemap::Terrain => entity_commands.insert(TerrainTilemap),
            Tilemap::Organisms => entity_commands.insert(OrganismsTilemap),
            Tilemap::Produce => entity_commands.insert(ProduceTilemap),
        };

        entity_commands.id()
    }
}

/// Manages the mapping between [`Tilemap`]s and `bevy_ecs_tilemap` tilemaps
#[derive(Resource, Default, Debug)]
pub struct TilemapRegister {
    /// A vector consisting of the tilemaps initialized with each corresponding [`Tilemap`] variant.
    pub register: Vec<TilemapId>,
}
