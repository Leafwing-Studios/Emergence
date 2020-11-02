use bevy::prelude::*;
use itertools::Itertools as _;
use rand::prelude::{IteratorRandom, SliceRandom};
use std::convert::TryInto;

use crate::utils::Position;
use crate::units::build_hive;
use crate::structures::{build_plant, build_fungi};
use crate::terrain::build_tile;

use crate::config::MAP_SIZE;
use crate::config::{N_HIVE, N_PLANT, N_FUNGI};

#[derive(Copy, Clone)]
pub struct GenerationConfig {
	pub map_size: isize,
	n_hive: usize,
	n_plant: usize,
	n_fungi: usize
}

impl GenerationConfig {
	fn new() -> GenerationConfig {
		assert!((N_HIVE + N_PLANT + N_FUNGI) <= (MAP_SIZE * MAP_SIZE).try_into().unwrap());
		GenerationConfig{
			map_size : MAP_SIZE,
			n_hive : N_HIVE,
			n_plant : N_PLANT,
			n_fungi : N_FUNGI
		}
	}
}

pub struct GenerationPlugin;
impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut AppBuilder) {
		app
			.add_resource(GenerationConfig::new())
			.add_startup_system(generate_terrain.thread_local_system())
        	.add_startup_system(generate_entities.thread_local_system());
    }
}

fn generate_terrain(world: &mut World, resources: &mut Resources) {
    println!("Generating terrain.");

    let config = *resources.get::<GenerationConfig>().unwrap();
    let map_size = config.map_size;

    assert!(map_size > 0);

    let positions = (0..map_size).cartesian_product(0..map_size);

    let tiles = positions.map(|(x, y)| (build_tile(Position{x, y})));

    world.spawn_batch(tiles);
    println!("Terrain generated.");
}

fn generate_entities(world: &mut World, resources: &mut Resources) {
    println!("Generating entities.");

    let config = *resources.get::<GenerationConfig>().unwrap();
	
	let n_hive = config.n_hive;
	let n_plant = config.n_plant;
	let n_fungi = config.n_fungi;

	let n_entities = n_hive + n_plant + n_fungi;

	let map_size = config.map_size;

    let possible_positions = (0..map_size).cartesian_product(0..map_size);
    let mut rng = &mut rand::thread_rng();
    let mut entity_positions = possible_positions.choose_multiple(&mut rng, n_entities);
	entity_positions.shuffle(&mut rng);

	let build_pairs: &[(fn (Position), usize)] = &[
		(build_hive, n_hive),
		(build_plant, n_plant),
		(build_fungi, n_fungi),
	];

	let mut iter = entity_positions.into_iter();
	for (build, n) in build_pairs {
		let build_iter = iter.by_ref().take(*n).map(|(x, y)| build(Position{x, y}));
		world.spawn_batch(build_iter);
	}

	println!("Entities generated.");
}




