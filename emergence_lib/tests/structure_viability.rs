use bevy::prelude::*;
use emergence_lib::organisms::structures::Structure;
use emergence_lib::terrain::generation::GenerationConfig;

fn single_structure_app(generation_config: GenerationConfig) -> App {
    let mut app = emergence_lib::testing::simulation_app();
    app.insert_resource(generation_config);

    // Run startup systems
    app.update();
    app
}

/// How long should we wait to see if things have died off?
const N_CYCLES: usize = 500;

#[test]
fn plants_are_self_sufficient() {
    let generation_config = GenerationConfig {
        map_radius: 1,
        n_ant: 0,
        n_plant: 1,
        n_fungi: 0,
    };

    let mut app = single_structure_app(generation_config);

    let mut plant_query = app.world.query::<&Structure>();
    let n_plants = plant_query.iter(&app.world).len();
    assert_eq!(n_plants, 1);

    for _ in 0..N_CYCLES {
        app.update();
    }

    let mut plant_query = app.world.query::<&Structure>();
    let n_plants = plant_query.iter(&app.world).len();
    assert_eq!(n_plants, 1);
}
