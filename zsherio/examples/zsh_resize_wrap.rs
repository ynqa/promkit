use std::{thread, time::Duration};

use zsherio::{
    opts::clear_screen_and_move_cursor_to,
    scenarios::resize_wrap::{TERMINAL_COLS, TERMINAL_ROWS, scenario},
    session::{spawn_session_with_cursor, spawn_zsh_session},
};

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    // Before create scenaro, move cursor to bottom.
    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    let run = scenario().run("zsh", &mut session)?;
    run.write_to_stdout()
}
