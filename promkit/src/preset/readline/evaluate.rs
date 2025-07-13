use promkit_widgets::{listbox::Listbox, text::Text, text_editor};

use crate::{
    core::crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    preset::readline::{Focus, Readline},
    Signal,
};

pub async fn default(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match ctx.focus {
        Focus::Readline => {
            // Handle the readline input events.
            return readline(event, ctx).await;
        }
        Focus::Suggestion => {
            // Handle the suggestion input events.
            return suggestion(event, ctx).await;
        }
    }
}

/// Default key bindings for the text editor.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the editor if input is valid, otherwise show error message
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>←</kbd>           | Move the cursor one character to the left
/// | <kbd>→</kbd>           | Move the cursor one character to the right
/// | <kbd>Ctrl + A</kbd>    | Move the cursor to the start of the line
/// | <kbd>Ctrl + E</kbd>    | Move the cursor to the end of the line
/// | <kbd>↑</kbd>           | Recall the previous entry from history
/// | <kbd>↓</kbd>           | Recall the next entry from history
/// | <kbd>Backspace</kbd>   | Delete the character before the cursor
/// | <kbd>Ctrl + U</kbd>    | Delete all characters in the current line
/// | <kbd>Tab</kbd>         | Autocomplete the current input based on available suggestions
/// | <kbd>Alt + B</kbd>     | Move the cursor to the previous nearest character within set (default: whitespace)
/// | <kbd>Alt + F</kbd>     | Move the cursor to the next nearest character within set (default: whitespace)
/// | <kbd>Ctrl + W</kbd>    | Erase to the previous nearest character within set (default: whitespace)
/// | <kbd>Alt + D</kbd>     | Erase to the next nearest character within set (default: whitespace)
pub async fn readline(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let text = ctx.readline.texteditor.text_without_cursor().to_string();
            let valid = ctx
                .validator
                .as_ref()
                .map(|validator| {
                    let valid = validator.validate(&text);
                    if !valid {
                        ctx.error_message.text =
                            Text::from(validator.generate_error_message(&text));
                    }
                    valid
                })
                .unwrap_or(true);
            return {
                if valid {
                    if let Some(ref mut history) = &mut ctx.readline.history {
                        history.insert(text);
                    }
                    // For representing the end of the prompt,
                    // reset the style of the cursor to default.
                    ctx.readline.active_char_style = ContentStyle::default();
                    Ok(Signal::Quit)
                } else {
                    Ok(Signal::Continue)
                }
            };
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(anyhow::anyhow!("ctrl+c")),

        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(suggest) = &ctx.suggest {
                let text = ctx.readline.texteditor.text_without_cursor().to_string();
                if let Some(candidates) = suggest.prefix_search(text) {
                    ctx.suggestions.listbox = Listbox::from_displayable(candidates);
                    ctx.readline
                        .texteditor
                        .replace(&ctx.suggestions.listbox.get().to_string());

                    // Enter suggestion mode.
                    ctx.focus = Focus::Suggestion;
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
            ctx.readline.texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readline.texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.move_to_tail(),

        // Move cursor to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .move_to_previous_nearest(&ctx.readline.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .move_to_next_nearest(&ctx.readline.word_break_chars),

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.erase_all(),

        // Erase to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .erase_to_previous_nearest(&ctx.readline.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .erase_to_next_nearest(&ctx.readline.word_break_chars),

        // Choose history
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut ctx.readline.history {
                if history.backward() {
                    ctx.readline.texteditor.replace(&history.get())
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut ctx.readline.history {
                if history.forward() {
                    ctx.readline.texteditor.replace(&history.get())
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
        }) => match ctx.readline.edit_mode {
            text_editor::Mode::Insert => ctx.readline.texteditor.insert(*ch),
            text_editor::Mode::Overwrite => ctx.readline.texteditor.overwrite(*ch),
        },

        _ => (),
    }
    Ok(Signal::Continue)
}

pub async fn suggestion(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(anyhow::anyhow!("ctrl+c")),

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
            ctx.suggestions.listbox.forward();
            ctx.readline
                .texteditor
                .replace(&ctx.suggestions.listbox.get().to_string());
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.suggestions.listbox.backward();
            ctx.readline
                .texteditor
                .replace(&ctx.suggestions.listbox.get().to_string());
        }

        _ => {
            ctx.suggestions.listbox = Listbox::from_displayable(Vec::<String>::new());

            // Switch focus back to the readline input.
            ctx.focus = Focus::Readline;
        }
    }
    Ok(Signal::Continue)
}
