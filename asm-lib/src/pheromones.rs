use bevy::prelude::*;
use crate::config::PHEROMONE_CAPACITY;

pub struct PheromonesPlugin;
impl Plugin for PheromonesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // FIXME: add resource for current pheromone levels
        app.add_resource(PHEROMONE_CAPACITY);
    }
}

// pheromone_supply = PHEROMONE_CAPACITY;
