use promkit::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    text_editor,
};

use tokio::sync::mpsc::Sender;

use promkit_async::WrappedEvent;

pub type Handler = fn(&[WrappedEvent], &mut text_editor::State, &Sender<()>) -> anyhow::Result<()>;

pub fn default(
    events: &[WrappedEvent],
    state: &mut text_editor::State,
    fin_sender: &Sender<()>,
) -> anyhow::Result<()> {
    for event in events {
        match event {
            WrappedEvent::KeyBuffer(chars) => match state.edit_mode {
                text_editor::Mode::Insert => state.texteditor.insert_chars(&chars),
                text_editor::Mode::Overwrite => state.texteditor.overwrite_chars(&chars),
            },
            WrappedEvent::HorizontalCursorBuffer(left, right) => {
                state.texteditor.shift(*left, *right);
            }
            WrappedEvent::Others(e, times) => match e {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => {
                    fin_sender.try_send(())?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => state.texteditor.move_to_head(),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => state.texteditor.move_to_tail(),

                // Erase char(s).
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => {
                    for _ in 0..*times {
                        state.texteditor.erase();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => state.texteditor.erase_all(),
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
