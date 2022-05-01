use std::collections::HashMap;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    handler,
    keybind::KeyBind,
    readline::{self, State},
};

/// Default key bindings for readline.
///
/// | Key                    | Description
/// | :--                    | :--
/// | <kbd> Enter </kbd>     | Leave from event-loop with exitcode=0
/// | <kbd> CTRL + C </kbd>  | Leave from event-loop with [io::ErrorKind::Interrupted](https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html#variant.Interrupted)
/// | <kbd> ← </kbd>         | Move backward
/// | <kbd> → </kbd>         | Move forward
/// | <kbd> CTRL + A </kbd>  | Move to head of the input buffer
/// | <kbd> CTRL + E </kbd>  | Move to tail of the input buffer
/// | <kbd> ↑ </kbd>         | Look up a previous input in history
/// | <kbd> ↓ </kbd>         | Look up a next input in history
/// | <kbd> Backspace </kbd> | Erase a char at the current position
/// | <kbd> CTRL + U </kbd>  | Erase all chars at the current line
/// | <kbd> TAB </kbd>       | Tab completion by searching an item from the suggestions
impl Default for KeyBind<State> {
    fn default() -> Self {
        let mut b = KeyBind {
            event_mapping: HashMap::default(),
            handle_input: Some(readline::handler::input_char()),
            handle_resize: Some(readline::handler::reload()),
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
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::move_left(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::move_right(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                readline::handler::move_head(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                readline::handler::move_tail(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::prev_history(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::next_history(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::erase_char(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                readline::handler::erase_all(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::NONE,
                }),
                readline::handler::complete(),
            ),
        ]);
        b
    }
}
