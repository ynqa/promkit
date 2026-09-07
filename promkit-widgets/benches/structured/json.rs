use std::{
    fs::{self, File},
    hint::black_box,
    io::BufReader,
    path::Path,
};

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

    group.bench_function("from_file/via_value", |b| {
        b.iter(|| black_box(from_file_via_value(black_box(path))));
    });
    group.bench_function("from_file/direct", |b| {
        b.iter(|| {
            let file = File::open(black_box(path)).unwrap();
            black_box(json::Document::from_reader(BufReader::new(file)).unwrap())
        });
    });

    group.bench_function("from_str/via_value", |b| {
        b.iter(|| black_box(from_str_via_value(black_box(input.as_str()))));
    });
    group.bench_function("from_str/direct", |b| {
        b.iter(|| black_box(json::Document::from_str(black_box(input.as_str())).unwrap()));
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

    state.document.tail();
    state.document.up();
    group.bench_function("selected_path/tail", |b| {
        b.iter(|| black_box(state.document.selected_path()));
    });

    state.document.head();
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

fn from_file_via_value(path: &Path) -> json::Document {
    let input = fs::read_to_string(path).unwrap();
    from_str_via_value(&input)
}

fn from_str_via_value(input: &str) -> json::Document {
    let values = parse(input);
    json::Document::new(values.iter())
}
