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
use promkit_core::Widget;
use promkit_widgets::table::{CsvOptions, Document, State};

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

    let input = fs::read_to_string(path).unwrap();
    group.bench_function("parse_and_document_build", |b| {
        b.iter(|| {
            black_box(
                Document::from_csv(
                    black_box(input.as_bytes()),
                    black_box(CsvOptions::default()),
                )
                .unwrap(),
            )
        });
    });

    let document = Document::from_csv(input.as_bytes(), CsvOptions::default()).unwrap();
    drop(input);
    let mut state = State::new(document);

    group.bench_function("viewport_projection/120x40", |b| {
        b.iter(|| black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)));
    });

    state.document.tail();
    group.bench_function("viewport_projection_at_tail/120x40", |b| {
        b.iter(|| black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)));
    });
    state.document.scroll_to_end();
    group.bench_function("viewport_projection_at_bottom_right/120x40", |b| {
        b.iter(|| black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)));
    });
    state.document.head();
    state.document.scroll_to_start();

    let mut forward = true;
    group.bench_function("vertical_cursor_only", |b| {
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

    let mut forward = true;
    group.bench_function("horizontal_scroll_only", |b| {
        b.iter(|| {
            let moved = if forward {
                state.document.scroll_right()
            } else {
                state.document.scroll_left()
            };
            forward = !forward;
            black_box(moved)
        });
    });

    state.document.head();
    state.document.scroll_to_start();
    let mut forward = true;
    group.bench_function("vertical_cursor_and_projection/120x40", |b| {
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

    let mut forward = true;
    group.bench_function("horizontal_scroll_and_projection/120x40", |b| {
        b.iter(|| {
            if forward {
                state.document.scroll_right();
            } else {
                state.document.scroll_left();
            }
            forward = !forward;
            black_box(state.create_graphemes_in_viewport(VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
        });
    });

    group.finish();
}

const VIEWPORT_WIDTH: u16 = 120;
const VIEWPORT_HEIGHT: u16 = 40;

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
