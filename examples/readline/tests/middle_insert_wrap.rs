use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

const TERMINAL_ROWS: u16 = 6;
const TERMINAL_COLS: u16 = 80;
const INITIAL_CURSOR_ROW: u16 = 6;
const INITIAL_CURSOR_COL: u16 = 1;
const INPUT_TEXT: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqr";
const INSERTED_TEXT: &str = "HELLOWORLD!!!!";
const LEFT_MOVES: usize = 20;

fn read_expected_screen(path: &PathBuf) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect())
}

fn render_screen(snapshot: &[u8], rows: u16, cols: u16) -> Vec<String> {
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(snapshot);
    parser
        .screen()
        .rows(0, cols)
        .map(|row| row.trim_end().to_string())
        .collect()
}

fn format_screen_dump(lines: &[String]) -> String {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| format!("r{i:02}: {:?}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_screen(snapshot: &[u8], rows: u16, cols: u16, expected: &[String]) {
    let actual = render_screen(snapshot, rows, cols);
    assert_eq!(
        actual,
        expected,
        "screen mismatch\nexpected:\n{}\nactual:\n{}",
        format_screen_dump(expected),
        format_screen_dump(&actual)
    );
}

fn spawn_readline() -> anyhow::Result<(
    Box<dyn portable_pty::Child + Send + Sync>,
    Box<dyn MasterPty + Send>,
    Box<dyn Write + Send>,
    Arc<Mutex<Vec<u8>>>,
    thread::JoinHandle<()>,
)> {
    let pty = native_pty_system();
    let pair = pty.openpty(PtySize {
        rows: TERMINAL_ROWS,
        cols: TERMINAL_COLS,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_readline"));
    cmd.env("TERM", "xterm-256color");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let master = pair.master;
    let output = Arc::new(Mutex::new(Vec::new()));
    let output_reader = Arc::clone(&output);
    let mut reader = master.try_clone_reader()?;
    let reader_thread = thread::spawn(move || {
        let mut buf = [0_u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => output_reader
                    .lock()
                    .expect("failed to lock output buffer")
                    .extend_from_slice(&buf[..n]),
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    let writer = master.take_writer()?;
    Ok((child, master, writer, output, reader_thread))
}

#[test]
fn middle_insert_wrap() -> anyhow::Result<()> {
    let case_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let expected = read_expected_screen(&case_dir.join("middle_insert_wrap.after.txt"))?;

    let (mut child, _master, mut writer, output, reader_thread) = spawn_readline()?;

    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    writer.write_all(cpr.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    writer.write_all(INPUT_TEXT.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(120));

    for _ in 0..LEFT_MOVES {
        writer.write_all(b"\x1b[D")?;
        writer.flush()?;
        thread::sleep(Duration::from_millis(20));
    }

    writer.write_all(INSERTED_TEXT.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(250));

    let snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(&snapshot, TERMINAL_ROWS, TERMINAL_COLS, &expected);

    writer.write_all(b"\r")?;
    writer.flush()?;
    drop(writer);

    let _ = child.wait()?;
    reader_thread.join().expect("reader thread panicked");
    Ok(())
}
