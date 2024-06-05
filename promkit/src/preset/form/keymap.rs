use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    preset, text_editor, PromptSignal,
};

pub type Keymap = fn(
    event: &Event,
    renderer: &mut preset::form::render::Renderer,
) -> anyhow::Result<PromptSignal>;

pub fn default(
    event: &Event,
    renderer: &mut preset::form::render::Renderer,
) -> anyhow::Result<PromptSignal> {
    let current_position = renderer.text_editor_states.position();

    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(PromptSignal::Quit),
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
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.text_editor_states.contents_mut()[current_position]
            .texteditor
            .move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.text_editor_states.contents_mut()[current_position]
            .texteditor
            .move_to_tail(),

        // Move cursor to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = renderer.text_editor_states.contents_mut()[current_position]
                .word_break_chars
                .clone();
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .move_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = renderer.text_editor_states.contents_mut()[current_position]
                .word_break_chars
                .clone();
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .move_to_next_nearest(&word_break_chars)
        }

        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.text_editor_states.contents_mut()[current_position]
            .texteditor
            .erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.text_editor_states.contents_mut()[current_position]
            .texteditor
            .erase_all(),

        // Erase to the nearest character.
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = renderer.text_editor_states.contents_mut()[current_position]
                .word_break_chars
                .clone();
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .erase_to_previous_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let word_break_chars = renderer.text_editor_states.contents_mut()[current_position]
                .word_break_chars
                .clone();
            renderer.text_editor_states.contents_mut()[current_position]
                .texteditor
                .erase_to_next_nearest(&word_break_chars)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.text_editor_states.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.text_editor_states.forward();
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
        }) => match renderer.text_editor_states.contents_mut()[current_position].edit_mode {
            text_editor::Mode::Insert => renderer.text_editor_states.contents_mut()
                [current_position]
                .texteditor
                .insert(*ch),
            text_editor::Mode::Overwrite => renderer.text_editor_states.contents_mut()
                [current_position]
                .texteditor
                .overwrite(*ch),
        },

        _ => (),
    }
    Ok(PromptSignal::Continue)
}
