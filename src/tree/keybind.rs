use std::collections::HashMap;
use std::io::Stdout;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    handler,
    keybind::KeyBind,
    tree::{self, State},
    treeview::TreeView,
};

/// Default key bindings for select.
///
/// | Key                    | Description
/// | :--                    | :--
/// | <kbd> CTRL + C </kbd>  | Leave from event-loop with [io::ErrorKind::Interrupted](https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html#variant.Interrupted)
/// | <kbd> SPACE </kbd>     | Toggle fold/unfold
/// | <kbd> ↑ </kbd>         | Move backward
/// | <kbd> ↓ </kbd>         | Move forward
impl Default for KeyBind<State> {
    fn default() -> Self {
        let mut b = KeyBind::<State> {
            event_mapping: HashMap::default(),
            handle_input: None,
            handle_resize: Some(handler::reload::<TreeView, tree::state::With, Stdout>()),
        };
        b.assign(vec![
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }),
                handler::enter(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                handler::interrupt(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }),
                tree::handler::toggle(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }),
                tree::handler::move_up(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }),
                tree::handler::move_down(),
            ),
        ]);
        b
    }
}
