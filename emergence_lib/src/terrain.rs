use crate::config::{MAP_CENTER, MAP_COORD_SYSTEM, MAP_RADIUS, TERRAIN_TILE_IMAP};
use crate::tiles::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::generate_hexagon;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTexture};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::{thread_rng, Rng};

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

impl IntoTile for TerrainType {
    fn tile_texture(&self) -> TileTexture {
        TileTexture((&TERRAIN_TILE_IMAP).get_index_of(self).unwrap() as u32)
    }

    fn tile_texture_path(&self) -> &'static str {
        (&TERRAIN_TILE_IMAP).get(self).unwrap()
    }
}

impl TerrainType {
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

pub fn generate_simple_random_terrain(
    commands: &mut Commands,
    tilemap_id: TilemapId,
    tile_storage: &mut TileStorage,
) {
    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&MAP_CENTER, MAP_COORD_SYSTEM),
        MAP_RADIUS,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(MAP_COORD_SYSTEM));

    let mut rng = thread_rng();
    for position in tile_positions {
        let terrain: TerrainType = rng.gen();
        let entity = terrain.create_entity(commands, tilemap_id, position);
        tile_storage.set(&position, entity);
    }
}
