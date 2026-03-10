use std::{thread, time::Duration};

use termharness::{session::Session, terminal::TerminalSize};
use zsh_reference_capture::capture::{
    move_cursor_left, move_cursor_to, print_screen, send_bytes, spawn_zsh_session,
};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const RESIZED_TERMINAL_COLS: u16 = 20;
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 30;

fn resize(session: &mut Session, cols: u16) -> anyhow::Result<()> {
    session.resize(TerminalSize::new(TERMINAL_ROWS, cols))?;
    thread::sleep(Duration::from_millis(120));
    print_screen(
        &format!("resize -> {cols} cols"),
        session,
        TERMINAL_ROWS as usize,
    );
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    thread::sleep(Duration::from_millis(300));
    print_screen("spawn", &session, TERMINAL_ROWS as usize);

    move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));
    print_screen("move cursor to bottom", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, b"\"ynqa is a software engineer\"\r")?;
    thread::sleep(Duration::from_millis(300));
    print_screen("run echo", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, b"this is terminal test suite!")?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session, TERMINAL_ROWS as usize);

    // IMPORTANT: Move the zsh edit cursor far enough left so that, at the
    // narrowest resized width, it is no longer sitting on the active input's
    // reflow boundary. Otherwise, growing the terminal back can reflow the
    // active input differently and push older wrapped output out of view.
    move_cursor_left(&mut session, TIMES_TO_MOVE_CURSOR_LEFT)?;
    thread::sleep(Duration::from_millis(200));
    print_screen("move cursor left", &session, TERMINAL_ROWS as usize);
    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        resize(&mut session, cols)?;
    }
    for cols in (RESIZED_TERMINAL_COLS + 1)..=TERMINAL_COLS {
        resize(&mut session, cols)?;
    }

    Ok(())
}
