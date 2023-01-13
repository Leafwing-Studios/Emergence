use bevy::prelude::*;
use bevy::utils::HashMap;
use emergence_lib::simulation::generation::GenerationConfig;
use emergence_lib::structures::Structure;
use emergence_lib::terrain::TerrainType;

fn single_structure_app(gen_config: GenerationConfig) -> App {
    let mut app = emergence_lib::testing::simulation_app(gen_config);

    // Run startup systems
    app.update();
    app
}
