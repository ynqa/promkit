use promkit_widgets::{
    listbox::{Listbox, ListboxHit},
    text::Text,
    text_editor::{self, TextEditorHit},
};

use crate::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
                MouseEvent, MouseEventKind,
            },
            style::ContentStyle,
        },
        ScreenPosition,
    },
    preset::readline::{Focus, Index, Readline},
    Signal,
};

pub async fn default(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    // Handle the common events for both readline and suggestion modes.
    match event {
        // Quit
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Err(anyhow::anyhow!("ctrl+c")),

        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            click(*column, *row, ctx);
            Ok(Signal::Continue)
        }

        // Mouse movement, release, drag, and scroll do not dismiss suggestions.
        Event::Mouse(_) => Ok(Signal::Continue),

        _ => match ctx.focus {
            // Handle the readline input events.
            Focus::Readline => readline(event, ctx).await,
            // Handle the suggestion input events.
            Focus::Suggestion => suggestion(event, ctx).await,
        },
    }
}

fn click(column: u16, row: u16, ctx: &mut Readline) {
    let renderer = ctx.renderer.clone();
    let Some(position) = renderer
        .as_ref()
        .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
    else {
        return;
    };

    match position.index {
        Index::Readline => {
            if let Some(TextEditorHit::Cursor { index }) =
                ctx.readline.hit_at(position.content_position())
            {
                ctx.readline.texteditor.move_to(index);
            }
        }
        Index::Suggestion => {
            if let Some(ListboxHit::Select { index }) =
                ctx.suggestions.hit_at(position.content_position())
            {
                select_suggestion(index, ctx);
            }
        }
        Index::Title | Index::ErrorMessage => {}
    }
}

fn select_suggestion(index: usize, ctx: &mut Readline) {
    if ctx.suggestions.listbox.move_to(index) {
        ctx.focus = Focus::Suggestion;
    }
}

fn apply_suggestion(ctx: &mut Readline) {
    if ctx.suggestions.listbox.selected().is_none() {
        dismiss_suggestions(ctx);
        return;
    }

    let suggestion = ctx.suggestions.listbox.get().to_string();
    ctx.readline.texteditor.replace(&suggestion);
    dismiss_suggestions(ctx);
}

fn dismiss_suggestions(ctx: &mut Readline) {
    ctx.suggestions.listbox = Listbox::default();
    ctx.focus = Focus::Readline;
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
/// | <kbd>Tab</kbd>         | Show suggestions for the current input
/// | <kbd>Alt + B</kbd>     | Move the cursor to the previous nearest character within set (default: whitespace)
/// | <kbd>Alt + F</kbd>     | Move the cursor to the next nearest character within set (default: whitespace)
/// | <kbd>Ctrl + W</kbd>    | Erase to the previous nearest character within set (default: whitespace)
/// | <kbd>Alt + D</kbd>     | Erase to the next nearest character within set (default: whitespace)
/// | Left click             | Move the cursor or highlight the clicked suggestion
pub async fn readline(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        // Return the input text when the validation passes.
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
                    ctx.readline.config.active_char_style = ContentStyle::default();
                    Ok(Signal::Quit)
                } else {
                    Ok(Signal::Continue)
                }
            };
        }

        // Try to autocomplete
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(suggest) = &ctx.suggest {
                let text = ctx.readline.texteditor.text_without_cursor().to_string();
                if let Some(candidates) = suggest.prefix_search(text) {
                    ctx.suggestions.listbox = Listbox::from(candidates);

                    if ctx.suggestions.listbox.is_empty() {
                        dismiss_suggestions(ctx);
                    } else {
                        // Enter suggestion mode without changing the current input.
                        ctx.focus = Focus::Suggestion;
                    }
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
            .move_to_previous_nearest(&ctx.readline.config.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .move_to_next_nearest(&ctx.readline.config.word_break_chars),

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
            .erase_to_previous_nearest(&ctx.readline.config.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .erase_to_next_nearest(&ctx.readline.config.word_break_chars),

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
        }) => match ctx.readline.config.edit_mode {
            text_editor::Mode::Insert => ctx.readline.texteditor.insert(*ch),
            text_editor::Mode::Overwrite => ctx.readline.texteditor.overwrite(*ch),
        },

        _ => (),
    }
    Ok(Signal::Continue)
}

pub async fn suggestion(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        // Apply the highlighted suggestion.
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            apply_suggestion(ctx);
        }

        // Move cursor in the suggestion list.
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
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.suggestions.listbox.backward();
        }

        // Keep suggestions visible until applying one or changing the input.
        _ => {
            let before = ctx.readline.texteditor.text_without_cursor();
            let signal = readline(event, ctx).await?;
            let after = ctx.readline.texteditor.text_without_cursor();

            if before != after {
                dismiss_suggestions(ctx);
            }
            return Ok(signal);
        }
    }
    Ok(Signal::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggest::Suggest;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn mouse(kind: MouseEventKind) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn context_with_suggestions() -> Readline {
        let mut ctx = Readline::default();
        ctx.readline.texteditor.replace("app");
        ctx.suggestions.listbox = Listbox::from(["apple", "applet"]);
        ctx.focus = Focus::Suggestion;
        ctx
    }

    #[tokio::test]
    async fn tab_opens_suggestions_without_applying_one() {
        let mut ctx = Readline::default().enable_suggest(Suggest::from_iter(["apple", "applet"]));
        ctx.readline.texteditor.replace("app");

        readline(&key(KeyCode::Tab), &mut ctx).await.unwrap();

        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );
        assert!(!ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Suggestion));
    }

    #[tokio::test]
    async fn keyboard_navigation_only_changes_the_highlight() {
        let mut ctx = context_with_suggestions();

        suggestion(&key(KeyCode::Down), &mut ctx).await.unwrap();

        assert_eq!(ctx.suggestions.listbox.selected(), Some(1));
        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );

        suggestion(&key(KeyCode::Up), &mut ctx).await.unwrap();

        assert_eq!(ctx.suggestions.listbox.selected(), Some(0));
        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );
    }

    #[tokio::test]
    async fn enter_applies_the_highlighted_suggestion() {
        let mut ctx = context_with_suggestions();
        ctx.suggestions.listbox.move_to(1);

        let signal = suggestion(&key(KeyCode::Enter), &mut ctx).await.unwrap();

        assert!(matches!(signal, Signal::Continue));
        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "applet"
        );
        assert!(ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Readline));
    }

    #[test]
    fn repeated_clicks_only_change_the_highlight() {
        let mut ctx = context_with_suggestions();

        select_suggestion(1, &mut ctx);
        select_suggestion(1, &mut ctx);

        assert_eq!(ctx.suggestions.listbox.selected(), Some(1));
        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );
        assert!(!ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Suggestion));
    }

    #[tokio::test]
    async fn mouse_events_keep_suggestions_visible() {
        let mut ctx = context_with_suggestions();

        for event in [
            mouse(MouseEventKind::Up(MouseButton::Left)),
            mouse(MouseEventKind::Moved),
            mouse(MouseEventKind::ScrollDown),
        ] {
            default(&event, &mut ctx).await.unwrap();
        }

        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );
        assert!(!ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Suggestion));
    }

    #[tokio::test]
    async fn cursor_movement_keeps_suggestions_visible() {
        let mut ctx = context_with_suggestions();

        suggestion(&key(KeyCode::Left), &mut ctx).await.unwrap();

        assert_eq!(ctx.readline.texteditor.position(), 2);
        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "app"
        );
        assert!(!ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Suggestion));
    }

    #[tokio::test]
    async fn changing_the_input_dismisses_suggestions() {
        let mut ctx = context_with_suggestions();

        suggestion(&key(KeyCode::Char('x')), &mut ctx)
            .await
            .unwrap();

        assert_eq!(
            ctx.readline.texteditor.text_without_cursor().to_string(),
            "appx"
        );
        assert!(ctx.suggestions.listbox.is_empty());
        assert!(matches!(ctx.focus, Focus::Readline));
    }
}
