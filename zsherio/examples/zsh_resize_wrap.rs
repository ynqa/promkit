use std::time::Duration;

use termharness::{session::Session, terminal::TerminalSize};
use zsherio::Scenario;
use zsherio::capture::{move_cursor_left, move_cursor_to, send_bytes, spawn_zsh_session};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const RESIZED_TERMINAL_COLS: u16 = 20;
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 30;

fn resize(session: &mut Session, cols: u16) -> anyhow::Result<()> {
    session.resize(TerminalSize::new(TERMINAL_ROWS, cols))
}

fn scenario() -> Scenario {
    let mut scenario = Scenario::new("zsh_resize_wrap")
        .step("spawn", Duration::from_millis(300), |_session| Ok(()))
        .step(
            "move cursor to bottom",
            Duration::from_millis(300),
            |session| move_cursor_to(session, TERMINAL_ROWS, 1),
        )
        .step("run echo", Duration::from_millis(300), |session| {
            send_bytes(session, b"\"ynqa is a software engineer\"\r")
        })
        .step("type text", Duration::from_millis(200), |session| {
            send_bytes(session, b"this is terminal test suite!")
        });

    // Move the cursor far enough left so resizes do not reflow the active input
    // across the visible boundary.
    scenario = scenario.step("move cursor left", Duration::from_millis(200), |session| {
        move_cursor_left(session, TIMES_TO_MOVE_CURSOR_LEFT)
    });
    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        scenario = scenario.step(
            format!("resize -> {cols} cols"),
            Duration::from_millis(120),
            move |session| resize(session, cols),
        );
    }
    for cols in (RESIZED_TERMINAL_COLS + 1)..=TERMINAL_COLS {
        scenario = scenario.step(
            format!("resize -> {cols} cols"),
            Duration::from_millis(120),
            move |session| resize(session, cols),
        );
    }

    scenario
}

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;
    let run = scenario().run("zsh", &mut session)?;
    run.write_to_stdout()
}
