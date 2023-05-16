use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, MapGeometry, TilePos, Volume},
    terrain::terrain_manifest::{Terrain, TerrainData, TerrainManifest},
    water::{update_water_depth, WaterTable},
};

fn criterion_benchmark(c: &mut Criterion) {
    const MAP_RADIUS: u32 = 100;

    let mut app = App::new();

    let mut map_geometry = MapGeometry::new(MAP_RADIUS);
    let mut water_table = WaterTable::default();
    let mut terrain_manifest = TerrainManifest::default();
    let porous = "porous".to_string();
    let dense = "dense".to_string();

    terrain_manifest.insert(
        porous.clone(),
        TerrainData {
            water_capacity: 0.8,
            ..Default::default()
        },
    );

    terrain_manifest.insert(
        dense.clone(),
        TerrainData {
            water_capacity: 0.2,
            ..Default::default()
        },
    );

    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        // Make sure we cover a range of heights
        let height = Height(tile_pos.x.max(0) as f32);
        let volume_per_tile = Volume(20.);
        let terrain_string = if tile_pos.y % 2 == 0 { &porous } else { &dense };
        let terrain_id = Id::<Terrain>::from_name(terrain_string.clone());

        let terrain_entity = app.world.spawn((tile_pos, height, terrain_id)).id();

        map_geometry.update_height(tile_pos, height);
        map_geometry.add_terrain(tile_pos, terrain_entity);
        water_table.add(tile_pos, volume_per_tile);
    }

    app.insert_resource(map_geometry);
    app.insert_resource(water_table);
    app.insert_resource(terrain_manifest);

    app.add_system(update_water_depth);
    // Run once to make sure system caches are populated
    app.update();

    c.bench_function("compute_water_depth", |b| b.iter(|| app.update()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
