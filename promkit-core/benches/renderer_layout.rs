//! Benchmarks the renderer's terminal-size-dependent layout without terminal I/O.
//!
//! Each iteration clones the `CreatedGraphemes` inputs before laying them out,
//! matching the snapshot cost paid by `Renderer::render`.

use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use promkit_core::{
    ContentPosition, CreatedGraphemes, WidgetLayout, WidthMode, grapheme::StyledGraphemes,
    render::RendererLayout,
};

const TERMINAL_HEIGHT: u16 = 40;

#[derive(Clone, Copy)]
enum TextKind {
    Ascii,
    Unicode,
}

impl TextKind {
    fn name(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::Unicode => "unicode",
        }
    }

    fn line(self) -> &'static str {
        match self {
            Self::Ascii => {
                "renderer layout keeps interactive terminal updates predictable across panes"
            }
            Self::Unicode => "端末レイアウトは全角文字とe\u{301}の表示幅も処理します",
        }
    }
}

fn renderer_layout(c: &mut Criterion) {
    content_size(c);
    terminal_width(c);
    pane_count(c);
    cursor_scrolling(c);
}

fn content_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_layout/content_size");

    for size in [1024, 64 * 1024, 1024 * 1024] {
        group.throughput(Throughput::Bytes(size as u64));

        for kind in [TextKind::Ascii, TextKind::Unicode] {
            for mode in [WidthMode::Wrap, WidthMode::Truncate] {
                let contents = vec![(0usize, created_fixture(size, kind, mode, Cursor::Middle))];
                let mut layout = RendererLayout::default();
                let mode_name = match mode {
                    WidthMode::Wrap => "wrap",
                    WidthMode::Truncate => "truncate",
                };

                group.bench_with_input(
                    BenchmarkId::new(format!("{}/{}", kind.name(), mode_name), size),
                    &contents,
                    |b, contents| {
                        b.iter(|| {
                            black_box(
                                layout
                                    .layout(black_box(contents.clone()), 80, TERMINAL_HEIGHT)
                                    .unwrap(),
                            )
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

fn terminal_width(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_layout/terminal_width");
    let contents = vec![(
        0usize,
        created_fixture(
            64 * 1024,
            TextKind::Unicode,
            WidthMode::Wrap,
            Cursor::Middle,
        ),
    )];
    group.throughput(Throughput::Bytes(64 * 1024));

    for width in [20, 80, 240] {
        let mut layout = RendererLayout::default();
        group.bench_with_input(BenchmarkId::from_parameter(width), &width, |b, &width| {
            b.iter(|| {
                black_box(
                    layout
                        .layout(black_box(contents.clone()), width, TERMINAL_HEIGHT)
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn pane_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_layout/pane_count");

    for pane_count in [1usize, 10, 100] {
        let contents = (0..pane_count)
            .map(|index| {
                (
                    index,
                    created_fixture(4 * 1024, TextKind::Ascii, WidthMode::Wrap, Cursor::Middle),
                )
            })
            .collect::<Vec<_>>();
        let mut layout = RendererLayout::default();
        group.throughput(Throughput::Bytes((pane_count * 4 * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(pane_count),
            &contents,
            |b, contents| {
                b.iter(|| {
                    black_box(
                        layout
                            .layout(
                                black_box(contents.clone()),
                                80,
                                u16::try_from(pane_count.max(TERMINAL_HEIGHT as usize)).unwrap(),
                            )
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

fn cursor_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_layout/cursor_scrolling");
    let created = created_fixture(64 * 1024, TextKind::Ascii, WidthMode::Wrap, Cursor::Head);
    let last_row = created
        .graphemes
        .to_string()
        .lines()
        .count()
        .saturating_sub(1);
    let mut layout = RendererLayout::default();
    let mut at_tail = false;

    group.throughput(Throughput::Bytes(64 * 1024));
    group.bench_function("head_to_tail", |b| {
        b.iter(|| {
            let mut frame = created.clone();
            frame.cursor = Some(ContentPosition {
                row: if at_tail { last_row } else { 0 },
                column: 0,
            });
            at_tail = !at_tail;

            black_box(
                layout
                    .layout(black_box([(0usize, frame)]), 80, TERMINAL_HEIGHT)
                    .unwrap(),
            )
        });
    });

    group.finish();
}

#[derive(Clone, Copy)]
enum Cursor {
    Head,
    Middle,
}

fn created_fixture(
    target_bytes: usize,
    kind: TextKind,
    width_mode: WidthMode,
    cursor: Cursor,
) -> CreatedGraphemes {
    let text = text_fixture(target_bytes, kind);
    let line_count = text.lines().count();
    let cursor_row = match cursor {
        Cursor::Head => 0,
        Cursor::Middle => line_count / 2,
    };

    CreatedGraphemes {
        graphemes: StyledGraphemes::from(text),
        layout: WidgetLayout {
            max_height: None,
            width_mode,
        },
        cursor: Some(ContentPosition {
            row: cursor_row,
            column: 8,
        }),
    }
}

fn text_fixture(target_bytes: usize, kind: TextKind) -> String {
    let line = kind.line();
    let mut text = String::with_capacity(target_bytes + line.len() + 1);

    while text.len() < target_bytes {
        text.push_str(line);
        text.push('\n');
    }
    let mut end = target_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    text
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = renderer_layout
}
criterion_main!(benches);
