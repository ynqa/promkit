use std::{thread, time::Duration};

use zsh_reference_capture::capture::{
    move_cursor_left, move_cursor_to, print_screen, send_bytes, spawn_zsh_session,
};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
const INSERTED_TEXT: &str = " and open source maintainer";
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    thread::sleep(Duration::from_millis(300));
    print_screen("spawn", &session, TERMINAL_ROWS as usize);

    move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));
    print_screen("move cursor to bottom", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, INPUT_TEXT.as_bytes())?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session, TERMINAL_ROWS as usize);

    move_cursor_left(&mut session, TIMES_TO_MOVE_CURSOR_LEFT)?;
    thread::sleep(Duration::from_millis(200));
    print_screen("move cursor left", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, INSERTED_TEXT.as_bytes())?;
    thread::sleep(Duration::from_millis(250));
    print_screen("insert text", &session, TERMINAL_ROWS as usize);

    Ok(())
}
