use std::io;

use crate::{internal::buffer::Buffer, readline::State, termutil, Action};

/// Move the position of buffer and cursor backward.
pub fn move_left() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        let width = state.editor.width_in_position() as u16;
        if state.editor.prev() {
            termutil::move_left(out, width)?;
        }
        Ok(None)
    })
}

/// Move the position of buffer and cursor forward.
pub fn move_right() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        if state.editor.next() {
            termutil::move_right(out, state.editor.width_in_position() as u16)?;
        }
        Ok(None)
    })
}

/// Move the position of buffer and cursor to head.
pub fn move_head() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        termutil::move_left(out, state.editor.width_to_position() as u16)?;
        state.editor.to_head();
        Ok(None)
    })
}

/// Move the position of buffer and cursor to tail.
pub fn move_tail() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        termutil::move_right(out, state.editor.width_from_position() as u16)?;
        state.editor.to_tail();
        Ok(None)
    })
}

/// Look up a previous input in history.
pub fn prev_history() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.hstr {
            if hstr.prev() {
                state.editor.replace(&hstr.get());
            }
        }
        Ok(None)
    })
}

/// Look up a next input in history.
pub fn next_history() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.hstr {
            if hstr.next() {
                state.editor.replace(&hstr.get());
            }
        }
        Ok(None)
    })
}

/// Erase a char at the current position.
pub fn erase_char() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        if state.editor.position() > 0 {
            state.editor.erase();
        }
        Ok(None)
    })
}

/// Erase all chars at the current line.
pub fn erase_all() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        state.editor = Buffer::default();
        Ok(None)
    })
}

/// Search the item by [Suggest](../struct.Suggest.html).
pub fn complete() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        if let Some(suggest) = &state.suggest {
            if let Some(res) = suggest.search(&state.editor.data) {
                state.editor.replace(&res);
            }
        }
        Ok(None)
    })
}
