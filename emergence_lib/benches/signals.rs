use criterion::{criterion_group, criterion_main, Criterion};
use emergence_lib::asset_management::manifest::Id;
use emergence_lib::crafting::item_tags::ItemKind;
use emergence_lib::geometry::{MapGeometry, VoxelPos};
use emergence_lib::signals::{SignalStrength, SignalType, Signals, DIFFUSION_FRACTION};

/// Setup function
fn setup(settings: Settings) -> (Signals, MapGeometry) {
    let mut signals = Signals::default();
    let map_geometry = MapGeometry::new(settings.map_radius);

    for i in 0..settings.n_signals {
        let signal_type = SignalType::Pull(ItemKind::Single(Id::from_name(format!("{i}"))));

        for _ in 0..settings.n_sources {
            let voxel_pos = VoxelPos::ZERO;

            signals.add_signal(signal_type, voxel_pos, SignalStrength::new(1.));
        }
    }

    (signals, map_geometry)
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
    let (mut minimal_signals, minimal_map_geometry) = setup(Settings::MINIMAL);
    c.bench_function("signal_diffusion_minimal", |b| {
        b.iter(|| minimal_signals.diffuse(&minimal_map_geometry, DIFFUSION_FRACTION));
    });

    let (mut tiny_signals, tiny_map_geometry) = setup(Settings::TINY);
    c.bench_function("signal_diffusion_tiny", |b| {
        b.iter(|| tiny_signals.diffuse(&tiny_map_geometry, DIFFUSION_FRACTION));
    });

    let (mut modest_signals, modest_map_geometry) = setup(Settings::MODEST);
    c.bench_function("signal_diffusion_modest", |b| {
        b.iter(|| modest_signals.diffuse(&modest_map_geometry, DIFFUSION_FRACTION));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
