use promkit_widgets::text_editor;

use crate::{
    core::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    preset::form::Form,
    Signal,
};

/// Default event handler for the `Form` prompt.
pub async fn default(event: &Event, ctx: &mut Form) -> anyhow::Result<Signal> {
    let current_position = ctx.readlines.position();

    match event {
        // Resize the terminal.
        Event::Resize(width, height) => {
            ctx.render(*width, *height).await?;
        }

        // Quit the form.
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

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines.contents_mut()[current_position]
            .texteditor
            .move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines.contents_mut()[current_position]
            .texteditor
            .move_to_tail(),

        // Move cursor to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines.contents_mut()[current_position]
                .word_break_chars
                .clone();
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .move_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines.contents_mut()[current_position]
                .word_break_chars
                .clone();
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .move_to_next_nearest(&word_break_chars)
        }

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines.contents_mut()[current_position]
            .texteditor
            .erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readlines.contents_mut()[current_position]
            .texteditor
            .erase_all(),

        // Erase to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines.contents_mut()[current_position]
                .word_break_chars
                .clone();
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .erase_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = ctx.readlines.contents_mut()[current_position]
                .word_break_chars
                .clone();
            ctx.readlines.contents_mut()[current_position]
                .texteditor
                .erase_to_next_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readlines.forward();
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
        }) => match ctx.readlines.contents_mut()[current_position].edit_mode {
            text_editor::Mode::Insert => ctx.readlines.contents_mut()[current_position]
                .texteditor
                .insert(*ch),
            text_editor::Mode::Overwrite => ctx.readlines.contents_mut()[current_position]
                .texteditor
                .overwrite(*ch),
        },

        _ => (),
    }
    Ok(Signal::Continue)
}
