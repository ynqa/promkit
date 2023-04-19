use std::collections::HashMap;
use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    handler,
    internal::selector::Selector,
    keybind::KeyBind,
    select::{self, State},
    termutil,
};

/// Default key bindings for select.
///
/// | Key                    | Description
/// | :--                    | :--
/// | <kbd> Enter </kbd>     | Leave from event-loop with exitcode=0
/// | <kbd> CTRL + C </kbd>  | Leave from event-loop with [io::ErrorKind::Interrupted](https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html#variant.Interrupted)
/// | <kbd> ↑ </kbd>         | Move backward
/// | <kbd> ↓ </kbd>         | Move forward
/// | <kbd> CTRL + A </kbd>  | Move to head of the selectbox
/// | <kbd> CTRL + E </kbd>  | Move to tail of the selectbox
impl Default for KeyBind<State> {
    fn default() -> Self {
        let mut b = KeyBind::<State> {
            event_mapping: HashMap::default(),
            handle_input: None,
            handle_resize: Some(Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
                termutil::clear(out)?;
                state.editor.to_head();
                state.vertical_cursor.move_head()?;
                state.pre_render(out)?;
                // Overwrite the prev as default.
                state.prev = Selector::default();
                Ok(false)
            })),
        };
        b.assign(vec![
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                handler::enter(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                handler::interrupt(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::handler::move_up(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::handler::move_down(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::handler::move_head(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::handler::move_tail(),
            ),
        ]);
        b
    }
}
