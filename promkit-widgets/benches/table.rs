//! Benchmarks large tabular documents through the public widget API.
//!
//! The fixture defaults to `benches/table.csv`.
//! Override it with `PROMKIT_TABLE_CSV`.

use std::{
    env, fs,
    hint::black_box,
    path::{Path, PathBuf},
    time::Duration,
};

use criterion::{Criterion, criterion_group, criterion_main};

fn table(c: &mut Criterion) {
    if let Some(path) = fixture_path("PROMKIT_TABLE_CSV", "table.csv") {
        benchmark_csv(c, &path);
    }
}

fn benchmark_csv(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("table/csv");

    group.bench_function("read", |b| {
        b.iter(|| black_box(fs::read_to_string(black_box(path)).unwrap()));
    });

    group.finish();
}

fn fixture_path(environment_variable: &str, filename: &str) -> Option<PathBuf> {
    let path = env::var_os(environment_variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches")
                .join(filename)
        });

    if path.is_file() {
        Some(path)
    } else {
        eprintln!(
            "skipping {filename}: set {environment_variable} or place the fixture at {}",
            path.display()
        );
        None
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    targets = table
}
criterion_main!(benches);
