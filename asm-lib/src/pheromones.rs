use bevy::prelude::*;
use crate::config::PHEROMONE_CAPACITY;

pub struct PheromonesPlugin;
impl Plugin for PheromonesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_resource(PheromonesConfig{capacity: PHEROMONE_CAPACITY})
        .add_resource(Pheromones{supply: PHEROMONE_CAPACITY});
    }
}

struct Pheromones {
    supply: f32
}

struct PheromonesConfig {
    capacity: f32
}