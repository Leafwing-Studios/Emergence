use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::asset_management::manifest::Id;
use emergence_lib::crafting::item_tags::ItemKind;
use emergence_lib::signals::{SignalStrength, SignalType, Signals, DIFFUSION_FRACTION};
use emergence_lib::simulation::geometry::{MapGeometry, TilePos};
use rand::thread_rng;

/// Setup function
fn add_signals(settings: Settings) -> (Signals, MapGeometry) {
    let mut signals = Signals::default();
    let map_geometry = MapGeometry::new(settings.map_radius);
    let mut rng = thread_rng();

    for i in 0..settings.n_signals {
        let signal_type = SignalType::Pull(ItemKind::Single(Id::from_name(format!("{i}"))));

        for _ in 0..settings.n_sources {
            let tile_pos = TilePos::random(&map_geometry, &mut rng);

            signals.add_signal(signal_type, tile_pos, SignalStrength::new(1.));
        }
    }

    (signals, map_geometry)
}

/// Benchmarks the signal diffusion process
fn signal_diffusion(settings: Settings) {
    let (mut signals, map_geometry) = add_signals(settings);
    signals.diffuse(&map_geometry, DIFFUSION_FRACTION);
}

/// Benchmark settings, in a reusable form
struct Settings {
    map_radius: u32,
    n_signals: u64,
    n_sources: u32,
}

impl Settings {
    /// Controls how closely packed emitting structures are.
    ///
    /// Realistic values are between 2 and 20 or so.
    const SPARSITY: u32 = 5;

    const MINIMAL: Settings = Settings {
        map_radius: 1,
        n_signals: 1,
        n_sources: 1,
    };

    const TINY: Settings = Settings {
        map_radius: 10,
        n_signals: 10,
        n_sources: 10 * 10 / Self::SPARSITY,
    };

    const MODEST: Settings = Settings {
        map_radius: 100,
        n_signals: 100,
        n_sources: 100 * 100 / Self::SPARSITY,
    };

    // const HUGE: Settings = Settings {
    //     map_radius: 1000,
    //     n_signals: 1000,
    //     n_sources: 1000 * 1000 / Self::SPARSITY,
    // };
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("add_signal_minimal", |b| {
        b.iter(|| add_signals(Settings::MINIMAL))
    });
    c.bench_function("signal_diffusion_minimal", |b| {
        b.iter(|| signal_diffusion(Settings::MINIMAL))
    });

    c.bench_function("add_signal_tiny", |b| {
        b.iter(|| add_signals(Settings::TINY))
    });
    c.bench_function("signal_diffusion_tiny", |b| {
        b.iter(|| signal_diffusion(Settings::TINY))
    });

    c.bench_function("add_signal_modest", |b| {
        b.iter(|| add_signals(Settings::MODEST))
    });
    c.bench_function("signal_diffusion_modest", |b| {
        b.iter(|| signal_diffusion(Settings::MODEST))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
