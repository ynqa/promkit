//! Benchmarks large structured documents through the public widget API.
//!
//! Fixtures default to `benches/structured.json` and `benches/structured.yaml`.
//! Override them with `PROMKIT_STRUCTURED_JSON` and `PROMKIT_STRUCTURED_YAML`.

use std::{
    env, fs,
    hint::black_box,
    path::{Path, PathBuf},
    time::Duration,
};

use criterion::{Criterion, criterion_group, criterion_main};
use promkit_core::Widget;
use promkit_widgets::{json, yaml};
use serde::Deserialize;

const VIEWPORT_WIDTH: u16 = 120;
const VIEWPORT_HEIGHT: u16 = 40;

fn structured(c: &mut Criterion) {
    if let Some(path) = fixture_path("PROMKIT_STRUCTURED_JSON", "structured.json") {
        benchmark_json(c, &path);
    }
    if let Some(path) = fixture_path("PROMKIT_STRUCTURED_YAML", "structured.yaml") {
        benchmark_yaml(c, &path);
    }
}

fn benchmark_json(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("structured/json");

    group.bench_function("read", |b| {
        b.iter(|| black_box(fs::read_to_string(black_box(path)).unwrap()));
    });

    let input = fs::read_to_string(path).unwrap();
    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse_json(black_box(input.as_str()))));
    });

    let values = parse_json(&input);
    group.bench_function("document_build", |b| {
        b.iter(|| black_box(json::Document::new(black_box(values.iter()))));
    });

    let document = json::Document::new(values.iter());
    drop(values);
    drop(input);

    let mut state = json::State {
        document,
        config: json::Config::default(),
    };
    group.bench_function("viewport_projection/120x40", |b| {
        b.iter(|| black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)));
    });

    let mut forward = true;
    group.bench_function("cursor_only", |b| {
        b.iter(|| {
            let moved = if forward {
                state.document.down()
            } else {
                state.document.up()
            };
            forward = !forward;
            black_box(moved)
        });
    });

    state.document.head();
    let mut forward = true;
    group.bench_function("cursor_and_projection/120x40", |b| {
        b.iter(|| {
            if forward {
                state.document.down();
            } else {
                state.document.up();
            }
            forward = !forward;
            black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
        });
    });

    group.finish();
}

fn benchmark_yaml(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("structured/yaml");

    group.bench_function("read", |b| {
        b.iter(|| black_box(fs::read_to_string(black_box(path)).unwrap()));
    });

    let input = fs::read_to_string(path).unwrap();
    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse_yaml(black_box(input.as_str()))));
    });

    let values = parse_yaml(&input);
    group.bench_function("document_build", |b| {
        b.iter(|| black_box(yaml::Document::new(black_box(values.iter()))));
    });

    let document = yaml::Document::new(values.iter());
    drop(values);
    drop(input);

    let mut state = yaml::State {
        document,
        config: yaml::Config::default(),
    };
    group.bench_function("viewport_projection/120x40", |b| {
        b.iter(|| black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)));
    });

    let mut forward = true;
    group.bench_function("cursor_only", |b| {
        b.iter(|| {
            let moved = if forward {
                state.document.down()
            } else {
                state.document.up()
            };
            forward = !forward;
            black_box(moved)
        });
    });

    state.document.head();
    let mut forward = true;
    group.bench_function("cursor_and_projection/120x40", |b| {
        b.iter(|| {
            if forward {
                state.document.down();
            } else {
                state.document.up();
            }
            forward = !forward;
            black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
        });
    });

    group.finish();
}

fn parse_json(input: &str) -> Vec<serde_json::Value> {
    serde_json::Deserializer::from_str(input)
        .into_iter::<serde_json::Value>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn parse_yaml(input: &str) -> Vec<serde_yaml::Value> {
    serde_yaml::Deserializer::from_str(input)
        .map(serde_yaml::Value::deserialize)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
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
    targets = structured
}
criterion_main!(benches);
