use std::thread;
use std::time::Duration;

use zsherio::capture::{clear_screen_and_move_cursor_to, spawn_zsh_session};
use zsherio::scenarios::middle_insert_wrap::{TERMINAL_COLS, TERMINAL_ROWS, scenario};

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    // Before create scenaro, move cursor to bottom.
    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    let run = scenario().run("zsh", &mut session)?;
    run.write_to_stdout()
}
