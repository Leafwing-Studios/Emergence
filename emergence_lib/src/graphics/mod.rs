//! Utilities for defining and visualizing game graphics.

use crate::enum_iter::IterableEnum;
use crate::graphics::terrain::TerrainTilemap;

use bevy::app::{App, CoreStage, Plugin, StartupStage};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::{Res, ResMut, Resource};
use bevy_ecs_tilemap::map::{HexCoordSystem, TilemapId, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;

use crate as emergence_lib;
use crate::graphics::debug::generate_debug_labels;
use crate::graphics::organisms::OrganismsTilemap;
use crate::map::MapGeometry;
use bevy::prelude::{Added, Changed, Or};
use emergence_macros::IterableEnum;

use crate::graphics::produce::ProduceTilemap;
use crate::graphics::sprite_like::SpriteEnum;
use crate::graphics::tilemap_marker::TilemapMarker;
use bevy_trait_query::One;

pub mod debug;
pub mod organisms;
pub mod position;
pub mod produce;
pub mod sprite_like;
pub mod terrain;
pub mod tilemap_marker;

/// All of the code needed to draw things on screen.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .init_resource::<TilemapRegister>()
            .init_resource::<MapGeometry>()
            .add_startup_system_to_stage(StartupStage::PreStartup, initialize_tilemaps)
            .add_startup_system_to_stage(StartupStage::PostStartup, generate_debug_labels)
            .add_startup_system_to_stage(CoreStage::First, update_sprites);
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
            .insert(Tilemap::Terrain.index(), TilemapId(entity));
    });
}

/// Update entities that have a newly added/changed component which implements [`SpriteEnum`] with
/// new `bevy_ecs_tilemap` [`TileBundle`](bevy_ecs_tilemap::tiles::TileBundle) information.
fn update_sprites(
    mut commands: Commands,
    into_sprite_query: Query<
        (Entity, &TilePos, One<&dyn SpriteEnum>),
        Or<(Added<&dyn SpriteEnum>, Changed<&dyn SpriteEnum>)>,
    >,
    tilemap_register: Res<TilemapRegister>,
) {
    for (entity, position, sprite) in into_sprite_query.iter() {
        commands
            .entity(entity)
            .insert(sprite.tile_bundle(*position, &tilemap_register));
    }
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
