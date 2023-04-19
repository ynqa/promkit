use std::{collections::HashMap, io, marker::PhantomData};

use crate::{
    cmd,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    internal::selector::Selector,
    keybind::KeyBind,
    select::{self, State},
    termutil, InputHandler, Output, ResizeHandler, Result,
};

#[derive(Default)]
pub struct Handler<S> {
    _phantom: PhantomData<S>,
}

impl Default for Handler<State> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl InputHandler<State> for Handler<State> {
    fn handle(
        &mut self,
        _: char,
        _: &mut io::Stdout,
        _: &mut State,
    ) -> Result<Option<<State as Output>::Output>> {
        Ok(None)
    }
}

impl ResizeHandler<State> for Handler<State> {
    fn handle(
        &mut self,
        _: (u16, u16),
        out: &mut io::Stdout,
        state: &mut State,
    ) -> Result<Option<<State as Output>::Output>> {
        termutil::clear(out)?;
        state.editor.to_head();
        state.cursor.to_head();
        state.pre_render(out)?;
        // Overwrite the prev as default.
        state.prev = Selector::default();
        Ok(None)
    }
}

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
                select::cmd::move_up(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::cmd::move_down(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::cmd::move_head(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }),
                select::cmd::move_tail(),
            ),
        ]);
        b
    }
}
