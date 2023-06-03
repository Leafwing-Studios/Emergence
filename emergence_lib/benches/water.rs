use std::time::Duration;

use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::{
    geometry::{Height, MapGeometry, Volume, VoxelPos},
    simulation::time::InGameTime,
    water::{
        ocean::Ocean, update_water_depth, water_dynamics::horizontal_water_movement, WaterBundle,
        WaterConfig, WaterVolume,
    },
};
use hexx::Hex;

fn criterion_benchmark(c: &mut Criterion) {
    const MAP_RADIUS: u32 = 100;

    let mut app = App::new();

    let mut map_geometry = MapGeometry::new(MAP_RADIUS);

    for hex in map_geometry.all_hexes().copied().collect::<Vec<Hex>>() {
        // Make sure we cover a range of heights
        let height = Height(hex.x.max(0) as f32);
        let voxel_pos = VoxelPos::new(hex, height);

        let water_volume = WaterVolume::new(Volume(20.));

        let terrain_entity = app
            .world
            .spawn((
                voxel_pos,
                WaterBundle {
                    water_volume,
                    ..Default::default()
                },
            ))
            .id();

        map_geometry.update_height(voxel_pos);
        map_geometry.add_terrain(voxel_pos, terrain_entity);
    }

    app.insert_resource(map_geometry);
    app.insert_resource(Ocean::default());
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
