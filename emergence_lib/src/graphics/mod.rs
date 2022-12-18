//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::TerrainTilemap;
use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::asset::AssetServer;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::{Res, ResMut, Resource};
use bevy::prelude::{StageLabel, SystemStage};
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapId, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;

use crate as emergence_lib;
use crate::graphics::organisms::OrganismsTilemap;
use debug_tools::debug_ui::*;
use emergence_macros::IterableEnum;

use crate::graphics::produce::ProduceTilemap;
use crate::graphics::sprites::IntoSprite;
use crate::graphics::tilemap_marker::TilemapMarker;
use crate::organisms::structures::{Fungi, Plant};
use crate::organisms::units::Ant;
use crate::simulation::map::MapGeometry;
use crate::terrain::components::{HighTerrain, PlainTerrain, RockyTerrain};
use bevy_trait_query::{ChangedOne, RegisterExt};

pub mod organisms;
pub mod produce;
pub mod sprites;
pub mod terrain;
pub mod tilemap_marker;
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
            .add_plugin(FrameTimeDiagnosticsPlugin)
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
            .add_startup_system_to_stage(GraphicsStage::DebugLabelGeneration, initialize_infotext)
            .add_system_to_stage(CoreStage::Update, change_infotext)
            .add_system_to_stage(CoreStage::PreUpdate, update_sprites);
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
/// new `bevy_ecs_tilemap` [`TileBundle`](bevy_ecs_tilemap::tiles::TileBundle) information.\
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

/// We use a hexagonal map with "pointy-topped" (row oriented) graphics, and prefer an axial coordinate
/// system instead of an offset-coordinate system.
pub const MAP_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
/// We are using a map with hexagonal graphics.
pub const MAP_TYPE: TilemapType = TilemapType::Hexagon(HexCoordSystem::Row);

/// Enumerates the different tilemaps we are organizing our graphics into
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, IterableEnum)]
pub enum Tilemap {
    /// Organisms tilemap
    Organisms,
    /// Terrain tilemap
    Terrain,
    /// Produce tilemap
    Produce,
}

impl Tilemap {
    /// Spawns tilemap component associated with each variant
    pub fn spawn(
        &self,
        commands: &mut Commands,
        map_geometry: &Res<MapGeometry>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        match self {
            Tilemap::Organisms => {
                let tilemap = OrganismsTilemap;
                tilemap.spawn(commands, map_geometry, asset_server)
            }
            Tilemap::Terrain => {
                let tilemap = TerrainTilemap;
                tilemap.spawn(commands, map_geometry, asset_server)
            }
            Tilemap::Produce => {
                let tilemap = ProduceTilemap;
                tilemap.spawn(commands, map_geometry, asset_server)
            }
        }
    }
}

/// Manages the mapping between [`Tilemap`]s and `bevy_ecs_tilemap` tilemaps
#[derive(Resource, Default, Debug)]
pub struct TilemapRegister {
    /// A vector consisting of the tilemaps initialized with each corresponding [`Tilemap`] variant.
    pub register: Vec<TilemapId>,
}
