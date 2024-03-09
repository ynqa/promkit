use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    listbox, text_editor, Error, EventAction, Result,
};

pub fn default_text_editor_keymap(
    renderer: &mut text_editor::Renderer,
    event: &Event,
) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Ok(EventAction::Continue),
        _ => text_editor::keymap::default_keymap(renderer, event),
    }
}

pub fn default_suggest_keymap(_: &mut listbox::Renderer, _: &Event) -> Result<EventAction> {
    Ok(EventAction::Continue)
}

pub fn on_suggest_text_editor_keymap(
    renderer: &mut text_editor::Renderer,
    event: &Event,
) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
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
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Ok(EventAction::Continue),
        _ => text_editor::keymap::default_keymap(renderer, event),
    }
}

pub fn on_suggest_suggest_keymap(
    renderer: &mut listbox::Renderer,
    event: &Event,
) -> Result<EventAction> {
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
            renderer.listbox.forward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.listbox.backward();
        }

        _ => (),
    }
    Ok(EventAction::Continue)
}
