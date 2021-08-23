use bevy::prelude::*;
use rand::prelude::SliceRandom;

use crate::position::Position;

use crate::config::MAP_SIZE;
use crate::config::{N_ANT, N_FUNGI, N_PLANT};
use crate::structures::{FungiBundle, PlantBundle};
use crate::terrain::TileBundle;
use crate::units::AntBundle;

pub struct GenerationPlugin;
impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<GenerationConfig>()
            .add_startup_system(generate_terrain.system())
            .add_startup_system(generate_entities.system());
    }
}

#[derive(Copy, Clone)]
pub struct GenerationConfig {
    pub map_size: isize,
    n_ant: usize,
    n_plant: usize,
    n_fungi: usize,
}

impl Default for GenerationConfig {
    fn default() -> GenerationConfig {
        GenerationConfig {
            map_size: MAP_SIZE,
            n_ant: N_ANT,
            n_plant: N_PLANT,
            n_fungi: N_FUNGI,
        }
    }
}

fn generate_terrain(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    asset_server: Res<AssetServer>,
) {
    let map_size = config.map_size;

    assert!(map_size > 0);

    let center = Position { alpha: 0, beta: 0 };
    let positions = Position::hexagon(center, config.map_size);
    let tile_handle = asset_server.get_handle("tile.png");

    for position in positions {
        commands.spawn_bundle(TileBundle::new(position, tile_handle.clone()));
    }
}

fn generate_entities(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    asset_server: Res<AssetServer>,
) {
    let n_ant = config.n_ant;
    let n_plant = config.n_plant;
    let n_fungi = config.n_fungi;

    let n_entities = n_ant + n_plant + n_fungi;

    let center = Position { alpha: 0, beta: 0 };
    let possible_positions = Position::hexagon(center, config.map_size);
    let mut rng = &mut rand::thread_rng();
    let mut entity_positions: Vec<_> = possible_positions
        .choose_multiple(&mut rng, n_entities)
        .cloned()
        .collect();

    // PERF: Swap this to spawn_batch

    // Ant
    let ant_handle = asset_server.get_handle("ant.png");
    let positions = entity_positions.split_off(entity_positions.len() - n_ant);
    for position in positions {
        commands.spawn_bundle(AntBundle::new(position, ant_handle.clone()));
    }

    // Plant
    let plant_handle = asset_server.get_handle("plant.png");
    let positions = entity_positions.split_off(entity_positions.len() - n_plant);
    for position in positions {
        commands.spawn_bundle(PlantBundle::new(position, plant_handle.clone()));
    }

    // Fungi
    let fungi_handle = asset_server.get_handle("fungi.png");
    let positions = entity_positions.split_off(entity_positions.len() - n_fungi);
    for position in positions {
        commands.spawn_bundle(FungiBundle::new(position, fungi_handle.clone()));
    }
}
