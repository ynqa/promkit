use std::{fs, hint::black_box, path::Path};

use criterion::Criterion;
use promkit_core::Widget;
use promkit_widgets::yaml;
use serde::Deserialize;

use super::common::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH, fixture_path};

pub fn benchmark(c: &mut Criterion) {
    let Some(path) = fixture_path("PROMKIT_STRUCTURED_YAML", "structured.yaml") else {
        return;
    };
    benchmark_fixture(c, &path);
}

fn benchmark_fixture(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("structured/yaml");

    group.bench_function("read", |b| {
        b.iter(|| black_box(fs::read_to_string(black_box(path)).unwrap()));
    });

    let input = fs::read_to_string(path).unwrap();
    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse(black_box(input.as_str()))));
    });

    let values = parse(&input);
    group.bench_function("document_build", |b| {
        b.iter(|| black_box(yaml::Document::new(black_box(values.iter()))));
    });

    group.bench_function("from_file/via_value", |b| {
        b.iter(|| black_box(from_file_via_value(black_box(path))));
    });
    // TODO: Add `from_file/direct` when `yaml::Document::from_reader` is available.

    group.bench_function("from_str/via_value", |b| {
        b.iter(|| black_box(from_str_via_value(black_box(input.as_str()))));
    });
    // TODO: Add `from_str/direct` when `yaml::Document::from_str` is available.

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

fn parse(input: &str) -> Vec<serde_yaml::Value> {
    serde_yaml::Deserializer::from_str(input)
        .map(serde_yaml::Value::deserialize)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn from_file_via_value(path: &Path) -> yaml::Document {
    let input = fs::read_to_string(path).unwrap();
    from_str_via_value(&input)
}

fn from_str_via_value(input: &str) -> yaml::Document {
    let values = parse(input);
    yaml::Document::new(values.iter())
}
