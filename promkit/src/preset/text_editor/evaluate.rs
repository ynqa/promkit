use promkit_widgets::{
    text::Text,
    text_editor::{self as text_editor_widget, TextEditorHit},
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
    preset::text_editor::{IndentContext, Index, TextEditor},
    Signal,
};

/// Default key bindings for the multiline text editor.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Insert a newline
/// | <kbd>Ctrl + D</kbd>    | Submit the complete buffer if valid
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>←</kbd>/<kbd>→</kbd> | Move one grapheme
/// | <kbd>↑</kbd>/<kbd>↓</kbd> | Move one logical row
/// | <kbd>Home</kbd>/<kbd>End</kbd> | Move to the current row boundary
/// | <kbd>Ctrl + A</kbd>/<kbd>Ctrl + E</kbd> | Move to the current row boundary
/// | <kbd>Ctrl + Home</kbd>/<kbd>Ctrl + End</kbd> | Move to the document boundary
/// | <kbd>Backspace</kbd>/<kbd>Delete</kbd> | Delete backward or forward
/// | <kbd>Ctrl + U</kbd>    | Clear the complete buffer
/// | <kbd>Alt + B</kbd>/<kbd>Alt + F</kbd> | Move by word boundary
/// | <kbd>Ctrl + W</kbd>/<kbd>Alt + D</kbd> | Delete by word boundary
/// | Left click             | Move the cursor
pub async fn default(event: &Event, ctx: &mut TextEditor) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(anyhow::anyhow!("ctrl+c")),

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return submit(ctx),

        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            click(*column, *row, ctx);
            return Ok(Signal::Continue);
        }

        Event::Mouse(_) => return Ok(Signal::Continue),

        _ => {}
    }

    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => insert_newline(ctx),

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.editor.texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.editor.texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.editor.texteditor.move_up();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.editor.texteditor.move_down();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.move_to_line_head(),

        Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.move_to_line_tail(),

        Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.move_to_tail(),

        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .editor
            .texteditor
            .move_to_previous_nearest(&ctx.editor.config.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .editor
            .texteditor
            .move_to_next_nearest(&ctx.editor.config.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.erase_forward(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.editor.texteditor.erase_all(),

        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .editor
            .texteditor
            .erase_to_previous_nearest(&ctx.editor.config.word_break_chars),

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .editor
            .texteditor
            .erase_to_next_nearest(&ctx.editor.config.word_break_chars),

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
        }) => match ctx.editor.config.edit_mode {
            text_editor_widget::Mode::Insert => ctx.editor.texteditor.insert(*ch),
            text_editor_widget::Mode::Overwrite => ctx.editor.texteditor.overwrite(*ch),
        },

        _ => {}
    }

    Ok(Signal::Continue)
}

fn insert_newline(ctx: &mut TextEditor) {
    let indentation = ctx.indenter.map(|indenter| {
        let text = ctx.editor.texteditor.text_without_cursor().to_string();
        indenter(IndentContext {
            text: &text,
            cursor: ctx.editor.texteditor.position(),
            position: ctx.editor.texteditor.logical_position(),
        })
    });

    ctx.editor.texteditor.insert_newline();
    if let Some(indentation) = indentation {
        let characters = indentation.chars().collect();
        ctx.editor.texteditor.insert_chars(&characters);
    }
}

/// Validates and submits the current editor buffer.
///
/// Custom evaluators can call this when their own completion policy decides
/// that the buffer is ready to return.
pub fn submit(ctx: &mut TextEditor) -> anyhow::Result<Signal> {
    let text = ctx.editor.texteditor.text_without_cursor().to_string();
    let valid = ctx
        .validator
        .as_ref()
        .map(|validator| {
            let valid = validator.validate(&text);
            if valid {
                ctx.error_message.text = Text::default();
            } else {
                ctx.error_message.text = Text::from(validator.generate_error_message(&text));
            }
            valid
        })
        .unwrap_or(true);

    if valid {
        ctx.editor.config.active_char_style = ContentStyle::default();
        Ok(Signal::Quit)
    } else {
        Ok(Signal::Continue)
    }
}

fn click(column: u16, row: u16, ctx: &mut TextEditor) {
    let Some(position) = ctx
        .renderer
        .as_ref()
        .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
    else {
        return;
    };

    if position.index != Index::Editor {
        return;
    }

    if let Some(TextEditorHit::Cursor { index }) = ctx.editor.hit_at(position.content_position()) {
        ctx.editor.texteditor.move_to(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{preset::text_editor::IndentContext, widgets::text_editor::TextPosition};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn lua_indent(context: IndentContext<'_>) -> String {
        assert_eq!(context.text, "if ready then");
        assert_eq!(context.cursor, 13);
        assert_eq!(context.position, TextPosition { row: 0, column: 13 });
        "    ".to_string()
    }

    #[tokio::test]
    async fn enter_inserts_a_newline() {
        let mut ctx = TextEditor::default();
        ctx.editor.texteditor.replace("ab");
        assert!(ctx.editor.texteditor.move_to(1));

        let signal = default(&key(KeyCode::Enter, KeyModifiers::NONE), &mut ctx)
            .await
            .unwrap();

        assert_eq!(
            ctx.editor.texteditor.text_without_cursor().to_string(),
            "a\nb"
        );
        assert!(matches!(signal, Signal::Continue));
    }

    #[tokio::test]
    async fn enter_inserts_content_dependent_indentation() {
        let mut ctx = TextEditor::default().indenter(lua_indent);
        ctx.editor.texteditor.replace("if ready then");

        let signal = default(&key(KeyCode::Enter, KeyModifiers::NONE), &mut ctx)
            .await
            .unwrap();

        assert_eq!(
            ctx.editor.texteditor.text_without_cursor().to_string(),
            "if ready then\n    "
        );
        assert!(matches!(signal, Signal::Continue));
    }

    #[tokio::test]
    async fn ctrl_d_submits_the_complete_buffer() {
        let mut ctx = TextEditor::default();
        ctx.editor.texteditor.replace("first\nsecond");

        let signal = default(&key(KeyCode::Char('d'), KeyModifiers::CONTROL), &mut ctx)
            .await
            .unwrap();

        assert!(matches!(signal, Signal::Quit));
    }

    #[tokio::test]
    async fn failed_validation_keeps_the_editor_open() {
        let mut ctx = TextEditor::default().validator(
            |text| text.lines().count() > 1,
            |_| "more lines required".to_string(),
        );
        ctx.editor.texteditor.replace("one line");

        let signal = default(&key(KeyCode::Char('d'), KeyModifiers::CONTROL), &mut ctx)
            .await
            .unwrap();

        assert!(matches!(signal, Signal::Continue));
        assert_eq!(
            ctx.error_message.text.items()[0].to_string(),
            "more lines required"
        );
    }

    #[tokio::test]
    async fn arrow_keys_move_between_logical_rows() {
        let mut ctx = TextEditor::default();
        ctx.editor.texteditor.replace("abc\nx");
        assert!(ctx.editor.texteditor.move_to(2));

        default(&key(KeyCode::Down, KeyModifiers::NONE), &mut ctx)
            .await
            .unwrap();
        assert_eq!(ctx.editor.texteditor.position(), 5);

        default(&key(KeyCode::Up, KeyModifiers::NONE), &mut ctx)
            .await
            .unwrap();
        assert_eq!(ctx.editor.texteditor.position(), 2);
    }

    #[tokio::test]
    async fn delete_joins_logical_rows() {
        let mut ctx = TextEditor::default();
        ctx.editor.texteditor.replace("ab\ncd");
        assert!(ctx.editor.texteditor.move_to(2));

        default(&key(KeyCode::Delete, KeyModifiers::NONE), &mut ctx)
            .await
            .unwrap();

        assert_eq!(
            ctx.editor.texteditor.text_without_cursor().to_string(),
            "abcd"
        );
    }

    #[tokio::test]
    async fn ctrl_c_interrupts_the_editor() {
        let mut ctx = TextEditor::default();

        let result = default(&key(KeyCode::Char('c'), KeyModifiers::CONTROL), &mut ctx).await;

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "ctrl+c");
    }
}
