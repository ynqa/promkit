use std::thread;
use std::time::Duration;

use zsherio::Scenario;
use zsherio::capture::{
    clear_screen_and_move_cursor_to, move_cursor_left, send_bytes, spawn_zsh_session,
};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
const INSERTED_TEXT: &str = " and open source maintainer";
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

fn scenario() -> Scenario {
    Scenario::new("middle_insert_wrap")
        .step("spawn", Duration::from_millis(300), |_session| Ok(()))
        .step("type text", Duration::from_millis(100), |session| {
            send_bytes(session, INPUT_TEXT.as_bytes())
        })
        .step("move cursor left", Duration::from_millis(100), |session| {
            move_cursor_left(session, TIMES_TO_MOVE_CURSOR_LEFT)
        })
        .step("insert text", Duration::from_millis(100), |session| {
            send_bytes(session, INSERTED_TEXT.as_bytes())
        })
}

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    // Before create scenaro, move cursor to bottom.
    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    let run = scenario().run("zsh", &mut session)?;
    run.write_to_stdout()
}
