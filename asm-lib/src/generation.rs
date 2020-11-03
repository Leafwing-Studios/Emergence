use bevy::prelude::*;
use itertools::Itertools as _;
use rand::prelude::{IteratorRandom, SliceRandom};
use std::convert::TryInto;

use crate::graphics::{AntMaterial, FungiMaterial, PlantMaterial, TileMaterial};
use crate::structures::{build_fungi, build_plant};
use crate::terrain::build_tile;
use crate::units::build_ant;
use crate::utils::Position;

use crate::config::MAP_SIZE;
use crate::config::{N_ANT, N_FUNGI, N_PLANT};

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

pub struct GenerationPlugin;
impl Plugin for GenerationPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(GenerationConfig::new())
			.add_startup_system(generate_terrain.system())
			.add_startup_system(generate_entities.system());
	}
}

fn generate_terrain(
	mut commands: Commands,
	config: Res<GenerationConfig>,
	tile_material: Res<TileMaterial>,
) {
	println!("Generating terrain.");

	let map_size = config.map_size;

	assert!(map_size > 0);

	let positions = (0..map_size).cartesian_product(0..map_size);

	let tiles = positions.map(|(x, y)| (build_tile(Position { x, y })));

	commands.spawn_batch(tiles);
	println!("Terrain generated.");
}

fn generate_entities(mut commands: Commands, config: Res<GenerationConfig>) {
	println!("Generating entities.");

	let n_ant = config.n_ant;
	let n_plant = config.n_plant;
	let n_fungi = config.n_fungi;

	let n_entities = n_ant + n_plant + n_fungi;

	let map_size = config.map_size;

	let possible_positions = (0..map_size).cartesian_product(0..map_size);
	let mut rng = &mut rand::thread_rng();
	let mut entity_positions = possible_positions.choose_multiple(&mut rng, n_entities);
	entity_positions.shuffle(&mut rng);

	macro_rules! build_entity {
		($n:ident, $build:ident) => {
			let positions = entity_positions.split_off(entity_positions.len() - $n);
			let build_iter = positions
				.into_iter()
				.map(move |(x, y)| $build(Position { x, y }));
			commands.spawn_batch(build_iter);
		};
	}

	build_entity!(n_ant, build_ant);
	build_entity!(n_plant, build_plant);
	build_entity!(n_fungi, build_fungi);

	println!("Entities generated.");
}
