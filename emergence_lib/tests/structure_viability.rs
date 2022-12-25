use bevy::prelude::*;
use bevy::utils::HashMap;
use emergence_lib::organisms::structures::Structure;
use emergence_lib::simulation::generation::GenerationConfig;
use emergence_lib::terrain::TerrainType;

fn single_structure_app(gen_config: GenerationConfig) -> App {
    let mut app = emergence_lib::testing::simulation_app(gen_config);

    // Run startup systems
    app.update();
    app
}

/// How long should we wait to see if things have died off?
const N_CYCLES: usize = 500;

#[test]
fn plants_are_self_sufficient() {
    // Only spawn flat graphics, to ensure that the structure can actually be spawned
    let mut terrain_type_weights: HashMap<TerrainType, f32> = HashMap::new();
    terrain_type_weights.insert(TerrainType::Plain, 1.0);

    let gen_config = GenerationConfig {
        map_radius: 1,
        n_ant: 0,
        n_plant: 1,
        n_fungi: 0,
        terrain_weights: terrain_type_weights,
    };

    let mut app = single_structure_app(gen_config);

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
