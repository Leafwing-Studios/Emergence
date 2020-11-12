use bevy::prelude::*;
use rand::prelude::SliceRandom;
use std::convert::TryInto;

use crate::structures::{build_fungi, build_plant, StructureConfig};
use crate::terrain::build_tile;
use crate::units::build_ant;
use crate::utils::Position;

use crate::config::MAP_SIZE;
use crate::config::{N_ANT, N_FUNGI, N_PLANT};

pub struct GenerationPlugin;
impl Plugin for GenerationPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(GenerationConfig::new())
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

impl GenerationConfig {
	fn new() -> GenerationConfig {
		assert!((N_ANT + N_PLANT + N_FUNGI) <= (MAP_SIZE * MAP_SIZE).try_into().unwrap());
		GenerationConfig {
			map_size: MAP_SIZE,
			n_ant: N_ANT,
			n_plant: N_PLANT,
			n_fungi: N_FUNGI,
		}
	}
}

fn generate_terrain(
	commands: &mut Commands,
	config: Res<GenerationConfig>,
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	let map_size = config.map_size;

	assert!(map_size > 0);

	let center = Position { alpha: 0, beta: 0 };
	let positions = Position::hexagon(center, config.map_size);
	let tile_handle = asset_server.get_handle("tile.png");

	for position in positions {
		let fresh_handle = materials.add(tile_handle.clone().into());
		build_tile(commands, fresh_handle, position);
	}
}

fn generate_entities(
	commands: &mut Commands,
	config: Res<GenerationConfig>,
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	structure_config: Res<StructureConfig>,
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

	// TODO: Figure out how to swap this to spawn_batch
	// The main challenge is figuring out how to add the extra components

	// Ant
	let ant_handle = asset_server.get_handle("ant.png");
	let positions = entity_positions.split_off(entity_positions.len() - n_ant);
	for position in positions {
		let fresh_handle = materials.add(ant_handle.clone().into());
		build_ant(commands, fresh_handle, position);
	}

	// Plant
	let plant_handle = asset_server.get_handle("plant.png");
	let positions = entity_positions.split_off(entity_positions.len() - n_plant);
	for position in positions {
		let fresh_handle = materials.add(plant_handle.clone().into());
		build_plant(commands, fresh_handle, position, &structure_config);
	}

	// Fungi
	let fungi_handle = asset_server.get_handle("fungi.png");
	let positions = entity_positions.split_off(entity_positions.len() - n_fungi);
	for position in positions {
		let fresh_handle = materials.add(fungi_handle.clone().into());
		build_fungi(commands, fresh_handle, position, &structure_config);
	}
}
