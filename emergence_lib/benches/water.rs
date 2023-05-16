use std::time::Duration;

use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::{
    simulation::{
        geometry::{Height, MapGeometry, TilePos, Volume},
        time::InGameTime,
    },
    water::{
        update_water_depth,
        water_dynamics::{horizontal_water_movement, SoilWaterFlowRate},
        SoilWaterCapacity, WaterConfig, WaterDepth, WaterTable, WaterVolume,
    },
};

fn criterion_benchmark(c: &mut Criterion) {
    const MAP_RADIUS: u32 = 100;

    let mut app = App::new();

    let mut map_geometry = MapGeometry::new(MAP_RADIUS);
    let mut water_table = WaterTable::default();

    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        // Make sure we cover a range of heights
        let height = Height(tile_pos.x.max(0) as f32);
        let volume_per_tile = Volume(20.);
        let soil_water_flow_rate = SoilWaterFlowRate(0.1);
        let soil_water_capacity = SoilWaterCapacity(0.5);

        let terrain_entity = app
            .world
            .spawn((
                tile_pos,
                height,
                WaterVolume::default(),
                WaterDepth::default(),
                soil_water_flow_rate,
                soil_water_capacity,
            ))
            .id();

        map_geometry.update_height(tile_pos, height);
        map_geometry.add_terrain(tile_pos, terrain_entity);
        water_table.add(tile_pos, volume_per_tile);
    }

    app.insert_resource(map_geometry);
    app.insert_resource(water_table);
    app.insert_resource(WaterConfig::IN_GAME);
    app.insert_resource(InGameTime::default());
    app.insert_resource(FixedTime::new(Duration::from_secs_f32(1. / 30.)));

    app.add_system(update_water_depth);
    // Run once to make sure system caches are populated
    app.update();

    c.bench_function("compute_water_depth", |b| b.iter(|| app.update()));

    let mut schedule = Schedule::default();
    schedule.add_system(horizontal_water_movement);
    app.world.add_schedule(schedule, CoreSchedule::Outer);

    c.bench_function("lateral_water_movement", |b| b.iter(|| app.update()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
