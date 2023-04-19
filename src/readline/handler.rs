use std::io;

use crate::{
    buffer::Buffer,
    grapheme::Grapheme,
    readline::{Mode, State},
    termutil, EventHandleFn,
};

/// Move the position of buffer and cursor backward.
pub fn move_left() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        let width = state.editor.width_in_position() as u16;
        if state.editor.prev() {
            termutil::move_left(out, width)?;
        }
        Ok(false)
    })
}

/// Move the position of buffer and cursor forward.
pub fn move_right() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        if state.editor.next() {
            termutil::move_right(out, state.editor.width_in_position() as u16)?;
        }
        Ok(false)
    })
}

/// Move the position of buffer and cursor to head.
pub fn move_head() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        termutil::move_left(out, state.editor.width_to_position() as u16)?;
        state.editor.to_head();
        Ok(false)
    })
}

/// Move the position of buffer and cursor to tail.
pub fn move_tail() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        termutil::move_right(out, state.editor.width_from_position() as u16)?;
        state.editor.to_tail();
        Ok(false)
    })
}

/// Look up a previous input in history.
pub fn prev_history() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.hstr {
            if hstr.prev() {
                state.editor.replace(&hstr.get());
            }
        }
        Ok(false)
    })
}

/// Look up a next input in history.
pub fn next_history() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.hstr {
            if hstr.next() {
                state.editor.replace(&hstr.get());
            }
        }
        Ok(false)
    })
}

/// Erase a char at the current position.
pub fn erase_char() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if state.editor.position() > 0 {
            state.editor.erase();
        }
        Ok(false)
    })
}

/// Erase all chars at the current line.
pub fn erase_all() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.editor = Buffer::default();
        Ok(false)
    })
}

/// Search the item by [Suggest](../struct.Suggest.html).
pub fn complete() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(suggest) = &state.suggest {
            if state.editor.data.len() >= state.min_len_to_search {
                if let Some(res) = suggest.search(&state.editor.data) {
                    state.editor.replace(&res);
                }
            }
        }
        Ok(false)
    })
}

/// Insert or overwrite a char at the current position.
pub fn input_char() -> Box<EventHandleFn<State>> {
    Box::new(
        |_, input: Option<char>, _: &mut io::Stdout, state: &mut State| {
            if let Some(limit) = state.buffer_limit()? {
                if limit <= state.editor.data.width() {
                    return Ok(false);
                }
            }
            if let Some(input) = input {
                match state.edit_mode {
                    Mode::Insert => state.editor.insert(Grapheme::from(input)),
                    Mode::Overwrite => state.editor.overwrite(Grapheme::from(input)),
                }
            }
            Ok(false)
        },
    )
}
