use std::collections::HashMap;

use crate::{
    cmd,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    keybind::KeyBind,
    select::{action, State},
};

/// Default key bindings for select.
///
/// | Key                    | Description
/// | :--                    | :--
/// | <kbd> Enter </kbd>     | Leave from event-loop with exitcode=0
/// | <kbd> CTRL + C </kbd>  | Leave from event-loop with [io::ErrorKind::Interrupted](https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html#variant.Interrupted)
/// | <kbd> ↑ </kbd>         | Move backward
/// | <kbd> ↓ </kbd>         | Move forward
/// | <kbd> CTRL + A </kbd>  | Move to head of selector
/// | <kbd> CTRL + E </kbd>  | Move to tail of selector
impl Default for KeyBind<State> {
    fn default() -> Self {
        let mut b = KeyBind::<State> {
            event_mapping: HashMap::default(),
        };
        b.assign(vec![
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                cmd::enter(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                cmd::interrupt(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                action::move_up(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                action::move_down(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                action::move_head(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                action::move_tail(),
            ),
        ]);
        b
    }
}
