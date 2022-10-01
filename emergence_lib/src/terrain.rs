use crate::config::{
    MAP_CENTER, MAP_COORD_SYSTEM, MAP_RADIUS, TERRAIN_HIGH_PNG, TERRAIN_IMPASSABLE_PNG,
    TERRAIN_PLAIN_PNG,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::generate_hexagon;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTexture};
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

#[derive(Clone, Copy)]
pub enum Terrain {
    Plain,
    Impassable,
    High,
}

pub const TERRAIN_CHOICES: [Terrain; 3] = {
    use Terrain::*;
    [Plain, Impassable, High]
};

impl Terrain {
    pub fn tile_texture(&self) -> TileTexture {
        TileTexture(*self as u32)
    }

    pub fn tile_texture_path(&self) -> &'static str {
        use Terrain::*;
        match self {
            Plain => TERRAIN_PLAIN_PNG,
            Impassable => TERRAIN_IMPASSABLE_PNG,
            High => TERRAIN_HIGH_PNG,
        }
    }

    pub fn create_entity(
        &self,
        commands: &mut Commands,
        tilemap_id: TilemapId,
        position: TilePos,
    ) -> Entity {
        let mut builder = commands.spawn();

        builder.insert_bundle(TileBundle {
            position,
            tilemap_id,
            texture: self.tile_texture(),
            ..Default::default()
        });
        match self {
            Terrain::Plain => {
                builder.insert(PlainTerrain);
            }
            Terrain::Impassable => {
                builder.insert(ImpassableTerrain);
            }
            Terrain::High => {
                builder.insert(HighTerrain);
            }
        }
        builder.id()
    }
}

impl Distribution<Terrain> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Terrain {
        let c: f32 = rng.gen();
        if c < 0.1 {
            Terrain::High
        } else if c < 0.2 {
            Terrain::Impassable
        } else {
            Terrain::Plain
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
        let terrain: Terrain = rng.gen();
        let entity = terrain.create_entity(commands, tilemap_id, position);
        tile_storage.set(&position, entity);
    }
}
