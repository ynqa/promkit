use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    Error, EventAction, Result,
};

use super::Mode;

/// Default key bindings for the text editor.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the editor
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>←</kbd>           | Move the cursor one character to the left
/// | <kbd>→</kbd>           | Move the cursor one character to the right
/// | <kbd>Ctrl + A</kbd>    | Move the cursor to the start of the line
/// | <kbd>Ctrl + E</kbd>    | Move the cursor to the end of the line
/// | <kbd>↑</kbd>           | Recall the previous entry from history
/// | <kbd>↓</kbd>           | Recall the next entry from history
/// | <kbd>Backspace</kbd>   | Delete the character before the cursor
/// | <kbd>Ctrl + U</kbd>    | Delete all characters in the current line
pub fn default_keymap(renderer: &mut super::Renderer, event: &Event) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(EventAction::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(Error::Interrupted("ctrl+c".into())),

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.move_to_tail(),

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.erase_all(),

        // Choose history
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut renderer.history {
                if history.backward() {
                    renderer.texteditor.replace(&history.get())
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut renderer.history {
                if history.forward() {
                    renderer.texteditor.replace(&history.get())
                }
            }
        }

        // Input char.
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
        }) => match renderer.edit_mode {
            Mode::Insert => renderer.texteditor.insert(*ch),
            Mode::Overwrite => renderer.texteditor.overwrite(*ch),
        },

        _ => (),
    }
    Ok(EventAction::Continue)
}
