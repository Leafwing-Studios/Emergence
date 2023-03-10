use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::asset_management::manifest::Id;
use emergence_lib::signals::{SignalStrength, SignalType, Signals, DIFFUSION_FRACTION};
use emergence_lib::simulation::geometry::{MapGeometry, TilePos};
use rand::thread_rng;

/// Setup function
fn add_signals(map_radius: u32, n_signals: u64, n_sources: u32) -> (Signals, MapGeometry) {
    let mut signals = Signals::default();
    let map_geometry = MapGeometry::new(map_radius);
    let mut rng = thread_rng();

    for i in 0..n_signals {
        let signal_type = SignalType::Pull(Id::new(i));

        for _ in 0..n_sources {
            let tile_pos = TilePos::random(&map_geometry, &mut rng);

            signals.add_signal(signal_type, tile_pos, SignalStrength::new(1.));
        }
    }

    (signals, map_geometry)
}

/// Benchmarks the signal diffusion process
fn signal_diffusion(map_radius: u32, n_signals: u64, n_sources: u32) {
    let (mut signals, map_geometry) = add_signals(map_radius, n_signals, n_sources);
    signals.diffuse(&map_geometry, DIFFUSION_FRACTION);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("add_signal_minimal", |b| b.iter(|| add_signals(1, 1, 1)));
    c.bench_function("signal_diffusion_minimal", |b| {
        b.iter(|| signal_diffusion(1, 1, 1))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
