use bevy::prelude::*;
use crate::utils::Position;
use itertools::Itertools as _;

use crate::config::MAP_SIZE;

#[derive(Copy, Clone)]
struct MapGeneration{
    map_size: isize
}

#[derive(Bundle, Debug)]
struct Tile{
    position: Position
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_resource(MapGeneration{map_size : MAP_SIZE})
        .add_startup_system(generate_terrain.thread_local_system())
        .add_system(render_terrain.system());
    }
}

fn generate_terrain(world: &mut World, resources: &mut Resources) {
    println!("Generating terrain.");

    let map_generation = *resources.get::<MapGeneration>().unwrap();
    let map_size = map_generation.map_size;

    assert!(map_size > 0);

    let positions = (0..map_size).cartesian_product(0..map_size);

    let tiles = positions.map(|(x, y)| (Tile{position: Position{x, y}}, ));


    world.spawn_batch(tiles);
    println!("Terrain generated.");
}

fn render_terrain(tiles: &Tile){    
    println!("Tile: ({}, {})", tiles.position.x, tiles.position.y);    
}