use crate::config::{MAP_CENTER, MAP_COORD_SYSTEM, MAP_RADIUS, TERRAIN_PNGS};
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::generate_hexagon;
use bevy_ecs_tilemap::tiles::{TileBundle, TileStorage, TileTexture};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

#[derive(Component, Clone, Copy)]
pub enum Terrain {
    Plain,
    Impassable,
    High,
}

pub const TERRAIN_CHOICES: [Terrain; 3] = [Terrain::Plain, Terrain::Impassable, Terrain::High];

impl Terrain {
    pub fn choose_random<R: Rng + ?Sized>(mut rng: &mut R) -> Terrain {
        *(TERRAIN_CHOICES
            .choose_weighted(&mut rng, |t| t.weight())
            .unwrap())
    }
}

impl From<&Terrain> for u32 {
    fn from(terrain: &Terrain) -> Self {
        match terrain {
            Terrain::Plain => 0,
            Terrain::Impassable => 1,
            Terrain::High => 2,
        }
    }
}

impl Terrain {
    pub fn weight(&self) -> f32 {
        match self {
            Terrain::Plain => 0.8,
            Terrain::Impassable => 0.1,
            Terrain::High => 0.1,
        }
    }

    pub fn tile_texture(&self) -> TileTexture {
        TileTexture(self.into())
    }

    pub fn tile_texture_path(&self) -> &'static str {
        TERRAIN_PNGS[u32::from(self) as usize]
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
        let terrain = Terrain::choose_random(&mut rng);
        let tile_entity = commands
            .spawn()
            .insert_bundle(TileBundle {
                position,
                tilemap_id,
                texture: terrain.tile_texture(),
                ..Default::default()
            })
            .insert(terrain)
            .id();
        tile_storage.set(&position, tile_entity);
    }
}
