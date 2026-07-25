use promkit_widgets::text_editor;

use crate::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
            MouseEvent, MouseEventKind,
        },
        ScreenPosition,
    },
    preset::form::Form,
    widgets::text_editor::TextEditorHit,
    Signal,
};

/// Default event handler for the `Form` prompt.
pub async fn default(event: &Event, ctx: &mut Form) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(Signal::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(anyhow::anyhow!("ctrl+c")),
        _ => {}
    }

    let Some(current_position) = ctx.active() else {
        return Ok(Signal::Continue);
    };

    match event {
        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines[current_position].texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines[current_position].texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines[current_position].texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines[current_position].texteditor.move_to_tail(),

        // Move cursor to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines[current_position]
                .config
                .word_break_chars
                .clone();
            ctx.readlines[current_position]
                .texteditor
                .move_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines[current_position]
                .config
                .word_break_chars
                .clone();
            ctx.readlines[current_position]
                .texteditor
                .move_to_next_nearest(&word_break_chars)
        }

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines[current_position].texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines[current_position].texteditor.erase_all(),

        // Erase to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines[current_position]
                .config
                .word_break_chars
                .clone();
            ctx.readlines[current_position]
                .texteditor
                .erase_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines[current_position]
                .config
                .word_break_chars
                .clone();
            ctx.readlines[current_position]
                .texteditor
                .erase_to_next_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.focus_previous();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.focus_next();
        }

        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            let renderer = ctx.renderer.clone();
            if let Some(position) = renderer.as_ref().and_then(|renderer| {
                renderer.hit_test(ScreenPosition {
                    row: *row,
                    column: *column,
                })
            }) {
                let field_index = position.index;
                if let Some(TextEditorHit::Cursor { index }) = ctx
                    .readlines
                    .get(field_index)
                    .and_then(|state| state.hit_at(position.content_position()))
                {
                    ctx.focus(field_index);
                    ctx.readlines[field_index].texteditor.move_to(index);
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
        }) => match ctx.readlines[current_position].config.edit_mode {
            text_editor::Mode::Insert => ctx.readlines[current_position].texteditor.insert(*ch),
            text_editor::Mode::Overwrite => {
                ctx.readlines[current_position].texteditor.overwrite(*ch)
            }
        },

        _ => (),
    }
    Ok(Signal::Continue)
}
