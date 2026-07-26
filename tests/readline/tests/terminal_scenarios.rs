use std::{
    thread,
    time::{Duration, Instant},
};

use portable_pty::CommandBuilder;
use termharness::{error::Result, scenario, session::Session};

#[test]
fn mid_buffer_insert_wrap() -> Result<()> {
    run(include_str!("scenarios/mid_buffer_insert_wrap.th"))
}

#[test]
fn prompt_initial_render_at_mid_screen() -> Result<()> {
    run(include_str!(
        "scenarios/prompt_initial_render_at_mid_screen.th"
    ))
}

#[test]
fn resize_roundtrip_wrap_reflow() -> Result<()> {
    run(include_str!("scenarios/resize_roundtrip_wrap_reflow.th"))
}

#[test]
fn titled_long_input_resize_roundtrip_clears_stale_rows() -> Result<()> {
    let command = CommandBuilder::new(env!("CARGO_BIN_EXE_titled-readline-fixture"));
    let mut session = Session::spawn(command, 10, 60, 0, 9)?;
    thread::sleep(Duration::from_millis(300));

    session.write_input(
        b"Terminal prompts should remain stable when the window shrinks and expands again",
    )?;

    // Start resizing as soon as the fixture begins processing the queued input,
    // mirroring a fast split-pane divider drag while renders are still in flight.
    let deadline = Instant::now() + Duration::from_secs(1);
    while !session
        .output()
        .windows(b"Terminal".len())
        .any(|window| window == b"Terminal")
    {
        assert!(
            Instant::now() < deadline,
            "fixture did not begin rendering the input"
        );
        thread::sleep(Duration::from_millis(1));
    }

    // Intentionally do not settle between widths: the regression depends on
    // rapid, incremental resize notifications rather than one atomic resize.
    for cols in (12..60).rev() {
        session.resize(10, cols)?;
    }
    for cols in 13..=60 {
        session.resize(10, cols)?;
    }
    thread::sleep(Duration::from_millis(300));

    assert_eq!(
        session.screen_snapshot(),
        vec![
            "                                                            ",
            "                                                            ",
            "                                                            ",
            "                                                            ",
            "                                                            ",
            "                                                            ",
            "                                                            ",
            "Hi!                                                         ",
            "❯❯ Terminal prompts should remain stable when the window shr",
            "inks and expands again                                      ",
        ]
    );

    session.terminate()
}

#[test]
fn tiny_viewport_overflow_wrap_scroll() -> Result<()> {
    run(include_str!(
        "scenarios/tiny_viewport_overflow_wrap_scroll.th"
    ))
}

fn run(document: &str) -> Result<()> {
    scenario::run_document(document)?;
    Ok(())
}
