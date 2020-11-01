use bevy::prelude::*;
use itertools::Itertools as _;
use rand::prelude::IteratorRandom;

use crate::utils::Position;
use crate::units::{Unit, Hive};
use crate::structures::{Structure, Plant, Fungi};

use crate::config::MAP_SIZE;
use crate::config::{N_HIVE, N_PLANT, N_FUNGI};

#[derive(Copy, Clone)]
pub struct InitConfig {
	pub map_size: isize,
	n_hive: usize,
	n_plant: usize,
	n_fungi: usize
}

impl InitConfig {

	fn new(&self){
		assert!(sum(N_HIVE, N_PLANT, N_FUNGI) <= MAP_SIZE * MAP_SIZE);
		InitConfig{
			map_size : MAP_SIZE,
			n_hive : N_HIVE,
			n_plant : N_PLANT,
			n_fungi : N_FUNGI
		}
	}
}

pub struct InitPlugin;
impl Plugin for InitPlugin {
    fn build(&self, app: &mut AppBuilder) {
		app
			.add_resource(InitConfig.new())
			.add_startup_system(generate_terrain.thread_local_system())
        	.add_startup_system(generate_entities.thread_local_system())
    }
}

fn generate_terrain(world: &mut World, resources: &mut Resources) {
    println!("Generating terrain.");

    let map_generation = *resources.get::<MapGeneration>().unwrap();
    let map_size = map_generation.map_size;

    assert!(map_size > 0);

    let positions = (0..map_size).cartesian_product(0..map_size);

    let tiles = positions.map(|(x, y)| (Tile{}, Position{x, y}));

    world.spawn_batch(tiles);
    println!("Terrain generated.");
}

fn generate_entities(world: &mut World, resources: &mut Resources) {
    println!("Generating entities.");

    let config = *resources.get::<InitConfig>().unwrap();
	
	let n_hive = config.n_hive;
	let n_plants = config.n_plants;
	let n_fungi = config.n_fungi;

	let n_entities = sum(n_hive, n_plants, n_fungi);

	let map_size = (*resources.get::<crate::terrain::MapGeneration>().unwrap()).map_size;

    let possible_positions = (0..map_size).cartesian_product(0..map_size);
    let mut rng = &mut rand::thread_rng();
    let positions = possible_positions.choose_multiple(&mut rng, n_entities).shuffle();
	
	// Hive generation

	let i = 0;
	let hive_positions = positions[i..(n_hive+i)];
	i += n_hive;

	let hive = hive_positions.into_iter().map(|(x,y)| (Unit{}, Hive{}, Position{x, y}));

    world.spawn_batch(hive);
    println!("Hive generated.");

	// Plant generation

	let i = 0;
	let plant_positions = positions[i..(n_plant+i)];
	i += n_plant;

	let plants = plant_positions.into_iter().map(|(x,y)| (Unit{}, Plant{}, Position{x, y}));

    world.spawn_batch(plants);
	println!("Plants generated.");
	

	// Fungi generation

	let i = 0;
	let fungi_positions = positions[i..(n_fungi+i)];
	i += n_plant;

	let fungi = fungi_positions.into_iter().map(|(x,y)| (Unit{}, Fungi{}, Position{x, y}));

    world.spawn_batch(fungi);
	println!("Fungi generated.");

}




