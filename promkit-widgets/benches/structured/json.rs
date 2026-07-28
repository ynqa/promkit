use std::{fs, hint::black_box, path::Path};

use criterion::Criterion;
use promkit_core::Widget;
use promkit_widgets::json;

use super::common::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH, fixture_path};

pub fn benchmark(c: &mut Criterion) {
    let Some(path) = fixture_path("PROMKIT_STRUCTURED_JSON", "structured.json") else {
        return;
    };
    benchmark_fixture(c, &path);
}

fn benchmark_fixture(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("structured/json");

    group.bench_function("read", |b| {
        b.iter(|| black_box(fs::read_to_string(black_box(path)).unwrap()));
    });

    let input = fs::read_to_string(path).unwrap();
    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse(black_box(input.as_str()))));
    });

    let values = parse(&input);
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

fn parse(input: &str) -> Vec<serde_json::Value> {
    serde_json::Deserializer::from_str(input)
        .into_iter::<serde_json::Value>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}
