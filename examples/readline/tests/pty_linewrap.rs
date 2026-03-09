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
const INITIAL_CURSOR_ROW: u16 = 6; // 1-based CPR
const INITIAL_CURSOR_COL: u16 = 1; // 1-based CPR
const RESIZED_TERMINAL_COLS: u16 = 72;

const BEFORE_TEXT: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqr";
const INSERTED_TEXT: &str = "HELLOWORLD!!!!";
const LEFT_MOVES: usize = 20;
const EXPECTED_RESULT: &str =
    "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxHELLOWORLD!!!!yzabcdefghijklmnopqr";

#[derive(Clone, Copy)]
enum InputEvent<'a> {
    Type(&'a str),
    Left(usize),
    Enter,
}

fn run_events(writer: &mut dyn Write, events: &[InputEvent<'_>]) -> anyhow::Result<()> {
    for event in events {
        match event {
            InputEvent::Type(text) => writer.write_all(text.as_bytes())?,
            InputEvent::Left(times) => {
                for _ in 0..*times {
                    writer.write_all(b"\x1b[D")?;
                }
            }
            InputEvent::Enter => writer.write_all(b"\r")?,
        }
        writer.flush()?;
        thread::sleep(Duration::from_millis(120));
    }
    Ok(())
}

fn read_expected_screen(path: &PathBuf) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let lines = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return Err(anyhow::anyhow!(
            "empty expected screen file: {}",
            path.display()
        ));
    }
    Ok(lines)
}

fn assert_screen(
    snapshot: &[u8],
    rows: u16,
    cols: u16,
    expected: &[String],
    expect_wrapped: bool,
    stage: &str,
) -> anyhow::Result<()> {
    let actual = render_screen(snapshot, rows, cols);
    let expected_dump = format_screen_dump(expected);
    let actual_dump = format_screen_dump(&actual);

    if std::env::var("PROMKIT_TEST_DUMP_SCREEN").ok().as_deref() == Some("1") {
        eprintln!("[{stage}] screen dump:");
        for (i, row) in actual.iter().enumerate() {
            eprintln!("r{i:02}: {row:?}");
        }
    }

    assert!(
        expected.len() <= actual.len(),
        "[{stage}] expected lines overflow: expected={}, actual={}\nexpected:\n{}\nactual:\n{}",
        expected.len(),
        actual.len(),
        expected_dump,
        actual_dump
    );

    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(
            actual[i], *exp,
            "[{stage}] row mismatch at {}: expected {:?}, got {:?}\nexpected:\n{}\nactual:\n{}",
            i, exp, actual[i], expected_dump, actual_dump
        );
    }

    // Ensure line-wrap behavior matches expectation.
    let prompt_row = actual
        .iter()
        .position(|row| row.starts_with("❯❯ "))
        .ok_or_else(|| anyhow::anyhow!("[{stage}] prompt row not found"))?;
    if expect_wrapped {
        let next_row = actual.get(prompt_row + 1).ok_or_else(|| {
            anyhow::anyhow!("[{stage}] row after prompt not found; wrapping did not happen")
        })?;
        assert!(
            !next_row.trim().is_empty(),
            "[{stage}] expected wrapped text on row {}\nexpected:\n{}\nactual:\n{}",
            prompt_row + 1,
            expected_dump,
            actual_dump
        );
    } else {
        if let Some(next_row) = actual.get(prompt_row + 1) {
            assert!(
                next_row.trim().is_empty(),
                "[{stage}] expected no wrapped text but got {:?} on row {}\nexpected:\n{}\nactual:\n{}",
                next_row,
                prompt_row + 1,
                expected_dump,
                actual_dump
            );
        }
    }

    Ok(())
}

fn render_screen(snapshot: &[u8], rows: u16, cols: u16) -> Vec<String> {
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(snapshot);
    parser
        .screen()
        .rows(0, cols)
        .map(|row| row.trim_end().to_string())
        .collect::<Vec<_>>()
}

fn format_screen_dump(lines: &[String]) -> String {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| format!("r{i:02}: {:?}", line))
        .collect::<Vec<_>>()
        .join("\n")
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
    let output = Arc::new(Mutex::new(Vec::<u8>::new()));
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
fn wraps_when_inserting_in_middle_with_bottom_cursor_start() -> anyhow::Result<()> {
    let case_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let before_file = case_dir.join("middle_insert_wrap.before.txt");
    let after_file = case_dir.join("middle_insert_wrap.after.txt");
    let expected_before = read_expected_screen(&before_file)?;
    let expected_after = read_expected_screen(&after_file)?;

    let (mut child, _master, mut writer, output, reader_thread) = spawn_readline()?;

    // crossterm asks CPR with ESC[6n on startup.
    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    writer.write_all(cpr.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    run_events(&mut writer, &[InputEvent::Type(BEFORE_TEXT)])?;
    thread::sleep(Duration::from_millis(250));
    let before_snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(
        &before_snapshot,
        TERMINAL_ROWS,
        TERMINAL_COLS,
        &expected_before,
        false,
        "before",
    )?;

    run_events(
        &mut writer,
        &[
            InputEvent::Left(LEFT_MOVES),
            InputEvent::Type(INSERTED_TEXT),
        ],
    )?;
    thread::sleep(Duration::from_millis(250));
    let after_snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(
        &after_snapshot,
        TERMINAL_ROWS,
        TERMINAL_COLS,
        &expected_after,
        true,
        "after",
    )?;

    run_events(&mut writer, &[InputEvent::Enter])?;
    drop(writer);

    let status = child.wait()?;
    reader_thread.join().expect("reader thread panicked");
    let all_output = output.lock().expect("failed to lock output buffer").clone();
    let full_text = String::from_utf8_lossy(&all_output);

    assert!(
        status.success(),
        "readline exited with code {}: {full_text:?}",
        status.exit_code()
    );
    assert!(
        full_text.contains(&format!("result: \"{EXPECTED_RESULT}\"")),
        "unexpected final output: {full_text:?}"
    );

    Ok(())
}

#[test]
fn does_not_duplicate_title_when_resizing_to_wrap_bottom_prompt() -> anyhow::Result<()> {
    let case_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let before_file = case_dir.join("resize_wrap.before.txt");
    let after_file = case_dir.join("resize_wrap.after.txt");
    let expected_before = read_expected_screen(&before_file)?;
    let expected_after = read_expected_screen(&after_file)?;

    let (mut child, master, mut writer, output, reader_thread) = spawn_readline()?;

    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    writer.write_all(cpr.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    run_events(&mut writer, &[InputEvent::Type(BEFORE_TEXT)])?;
    thread::sleep(Duration::from_millis(250));

    let before_snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(
        &before_snapshot,
        TERMINAL_ROWS,
        TERMINAL_COLS,
        &expected_before,
        false,
        "before-resize",
    )?;

    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        master.resize(PtySize {
            rows: TERMINAL_ROWS,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        thread::sleep(Duration::from_millis(150));
    }

    let resized_snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(
        &resized_snapshot,
        TERMINAL_ROWS,
        RESIZED_TERMINAL_COLS,
        &expected_after,
        true,
        "after-resize",
    )?;

    run_events(&mut writer, &[InputEvent::Enter])?;
    drop(writer);

    let status = child.wait()?;
    reader_thread.join().expect("reader thread panicked");
    let all_output = output.lock().expect("failed to lock output buffer").clone();
    let full_text = String::from_utf8_lossy(&all_output);

    assert!(
        status.success(),
        "readline exited with code {}: {full_text:?}",
        status.exit_code()
    );
    assert!(
        full_text.contains(&format!("result: \"{BEFORE_TEXT}\"")),
        "unexpected final output: {full_text:?}"
    );

    Ok(())
}
