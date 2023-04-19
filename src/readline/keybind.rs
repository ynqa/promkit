use std::collections::HashMap;
use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    handler,
    internal::buffer::Buffer,
    keybind::KeyBind,
    readline::{self, State},
    termutil,
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
            handle_resize: Some(Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
                termutil::clear(out)?;
                state.pre_render(out)?;
                // Overwrite the prev as default.
                state.prev = Buffer::default();
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
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::move_left(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::move_right(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::move_head(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::move_tail(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::prev_history(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::next_history(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::erase_char(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::erase_all(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                readline::handler::complete(),
            ),
        ]);
        b
    }
}
