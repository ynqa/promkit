use promkit::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::{Error, Result},
    preset::Readline,
    text_editor, EventAction,
};

pub fn default_keymap(renderer: &mut text_editor::Renderer, event: &Event) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.keymap.switch("alternative"),

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
        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.erase(),

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
            text_editor::Mode::Insert => renderer.texteditor.insert(*ch),
            text_editor::Mode::Overwrite => renderer.texteditor.overwrite(*ch),
        },

        _ => (),
    }
    Ok(EventAction::Continue)
}

pub fn alternative_keymap(
    renderer: &mut text_editor::Renderer,
    event: &Event,
) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.keymap.switch("default"),

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
        // Erase char(s).
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.texteditor.erase(),

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
        }) => {
            let ch = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap()
            } else {
                ch.to_uppercase().next().unwrap()
            };
            match renderer.edit_mode {
                text_editor::Mode::Insert => renderer.texteditor.insert(ch),
                text_editor::Mode::Overwrite => renderer.texteditor.overwrite(ch),
            }
        }

        _ => (),
    }
    Ok(EventAction::Continue)
}

fn main() -> Result {
    let mut p = Readline::default()
        .title("Feel free to fill in")
        .register_keymap("default", default_keymap)
        .register_keymap("alternative", alternative_keymap)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
