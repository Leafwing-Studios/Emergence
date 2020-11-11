use crate::config::{PHEROMONE_CAPACITY, PHEROMONE_REGEN_RATE, PHEROMONE_SPENDING_RATE};
use bevy::prelude::*;

pub struct PheromonesPlugin;
impl Plugin for PheromonesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(PheromonesConfig {
            capacity: PHEROMONE_CAPACITY,
            regen_rate: PHEROMONE_REGEN_RATE,
            spending_rate: PHEROMONE_SPENDING_RATE,
        })
        .add_resource(Pheromones {
            supply: PHEROMONE_CAPACITY,
        })
        .add_system(regen_pheromones.system())
        .add_system(spend_pheromones.system())
        .add_system(display_pheromones.system());
    }
}

struct Pheromones {
    supply: f32,
}

struct PheromonesConfig {
    capacity: f32,
    regen_rate: f32,
    spending_rate: f32,
}

fn regen_pheromones(
    mut pheromones: ResMut<Pheromones>,
    config: Res<PheromonesConfig>,
    time: Res<Time>,
) {
    pheromones.supply =
        (pheromones.supply + config.regen_rate * time.delta_seconds).min(config.capacity);
}

fn spend_pheromones(
    mut pheromones: ResMut<Pheromones>,
    config: Res<PheromonesConfig>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        let spent_pheromones = (config.spending_rate * time.delta_seconds).min(pheromones.supply);
        pheromones.supply -= spent_pheromones;
    }
}

fn display_pheromones(pheromones: Res<Pheromones>) {
    dbg!(pheromones.supply);
}
