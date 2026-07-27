use std::io;

use futures::StreamExt;
use promkit_widgets::{
    core::{
        crossterm::{
            cursor,
            event::{
                self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyEventState,
                KeyModifiers,
            },
            execute,
            terminal::{disable_raw_mode, enable_raw_mode},
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    text_editor::{self, Mode},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Editor,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::Show, event::DisableMouseCapture,).ok();
        disable_raw_mode().ok();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), cursor::Hide, event::EnableMouseCapture,)?;
    let _terminal_guard = TerminalGuard;

    let mut editor = text_editor::State {
        config: text_editor::Config {
            prefix: "❯❯❯ ".into(),
            continuation_prefix: "... ".into(),
            lines: Some(4),
            ..Default::default()
        },
        ..Default::default()
    };
    let renderer = SharedRenderer::new(
        Renderer::try_new_with_graphemes([(Index::Editor, editor.create_graphemes())], true)
            .await?,
    );
    let mut events = EventStream::new();

    while let Some(event) = events.next().await {
        let event = event?;
        if event.is_resize() {
            continue;
        }
        if handle_event(&event, &mut editor) {
            break;
        }
        renderer
            .update([(Index::Editor, editor.create_graphemes())])
            .render()
            .await?;
    }

    Ok(())
}

fn handle_event(event: &Event, editor: &mut text_editor::State) -> bool {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c' | 'd'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return true,
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => editor.texteditor.insert_newline(),
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            editor.texteditor.move_up();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            editor.texteditor.move_down();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => match editor.config.edit_mode {
            Mode::Insert => editor.texteditor.insert(*ch),
            Mode::Overwrite => editor.texteditor.overwrite(*ch),
        },
        _ => {}
    }

    false
}
