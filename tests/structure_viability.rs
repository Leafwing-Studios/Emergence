use emergence_lib::terrain::generation::{GenerationConfig, GenerationPlugin};

fn single_structure_app() -> App {
    app.add_plugins(MinimalPlugins)
        .add_plugin(GenerationPlugin)
        .add_plugin(StructuresPlugin);

    app.insert_resource(GenerationConfig {
        /// Radius of the map.
    map_radius: 1,
    /// Size of the map.
    map_size: TilemapSize::,
    /// Location of the center tile.
    map_center: TilePos,
    /// Initial number of ants.
    n_ant: usize,
    /// Initial number of plants.
    n_plant: usize,
    /// Initial number of fungi.
    n_fungi: usize,
    })
}
