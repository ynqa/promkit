use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    listbox::Listbox,
    preset, text_editor, Error, PromptSignal, Result,
};

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
/// | <kbd>TAB</kbd>         | Autocomplete the current input based on available suggestions
pub fn default(
    event: &Event,
    renderer: &mut preset::readline::render::Renderer,
) -> Result<PromptSignal> {
    let text_editor_after_mut = renderer.text_editor_snapshot.after_mut();
    let error_message_after_mut = renderer.error_message_snapshot.after_mut();
    let suggest_after_mut = renderer.suggest_snapshot.after_mut();

    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let text = text_editor_after_mut
                .texteditor
                .text_without_cursor()
                .to_string();
            let valid = renderer
                .validator
                .as_ref()
                .map(|validator| {
                    let valid = validator.validate(&text);
                    if !valid {
                        error_message_after_mut.text = validator.generate_error_message(&text);
                    }
                    valid
                })
                .unwrap_or(true);
            return {
                if valid {
                    if let Some(ref mut history) = &mut text_editor_after_mut.history {
                        history.insert(text);
                    }
                    Ok(PromptSignal::Quit)
                } else {
                    Ok(PromptSignal::Continue)
                }
            };
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(Error::Interrupted("ctrl+c".into())),

        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(suggest) = &renderer.suggest {
                let text = text_editor_after_mut
                    .texteditor
                    .text_without_cursor()
                    .to_string();
                if let Some(candidates) = suggest.prefix_search(text) {
                    suggest_after_mut.listbox = Listbox::from_iter(candidates);
                    text_editor_after_mut
                        .texteditor
                        .replace(&suggest_after_mut.listbox.get());

                    renderer.keymap.switch("on_suggest");
                }
            }
        }

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            text_editor_after_mut.texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            text_editor_after_mut.texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut.texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut.texteditor.move_to_tail(),

        // Move cursor to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut
            .texteditor
            .move_to_previous_nearest(&text_editor_after_mut.nearest_characters),

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut
            .texteditor
            .move_to_next_nearest(&text_editor_after_mut.nearest_characters),

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut.texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => text_editor_after_mut.texteditor.erase_all(),

        // Choose history
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut text_editor_after_mut.history {
                if history.backward() {
                    text_editor_after_mut.texteditor.replace(&history.get())
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut text_editor_after_mut.history {
                if history.forward() {
                    text_editor_after_mut.texteditor.replace(&history.get())
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
        }) => match text_editor_after_mut.edit_mode {
            text_editor::Mode::Insert => text_editor_after_mut.texteditor.insert(*ch),
            text_editor::Mode::Overwrite => text_editor_after_mut.texteditor.overwrite(*ch),
        },

        _ => (),
    }
    Ok(PromptSignal::Continue)
}

pub fn on_suggest(
    event: &Event,
    renderer: &mut preset::readline::render::Renderer,
) -> Result<PromptSignal> {
    let text_editor_after_mut = renderer.text_editor_snapshot.after_mut();
    let suggest_after_mut = renderer.suggest_snapshot.after_mut();

    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(Error::Interrupted("ctrl+c".into())),

        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            suggest_after_mut.listbox.forward();
            text_editor_after_mut
                .texteditor
                .replace(&suggest_after_mut.listbox.get());
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            suggest_after_mut.listbox.backward();
            text_editor_after_mut
                .texteditor
                .replace(&suggest_after_mut.listbox.get());
        }

        _ => {
            suggest_after_mut.listbox = Listbox::from_iter(Vec::<String>::new());

            renderer.keymap.switch("default");
        }
    }
    Ok(PromptSignal::Continue)
}
